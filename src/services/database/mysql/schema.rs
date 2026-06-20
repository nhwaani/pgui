//! MySQL schema introspection via `information_schema`.
//!
//! In MySQL "schema" and "database" are synonyms. We populate
//! `table_schema` with `TABLE_SCHEMA` for parity with the Postgres
//! backend; the active database is derived from `DATABASE()` so that
//! listings stay scoped to the connected DB.
//!
//! ## VARBINARY gotcha (MySQL 8 data dictionary)
//!
//! MySQL 8 reimplemented `information_schema` on top of the data
//! dictionary, and as a side effect many text columns (`TABLE_NAME`,
//! `COLUMN_NAME`, `INDEX_NAME`, etc.) come back as **`VARBINARY`**
//! over the wire instead of `VARCHAR`. sqlx will refuse to decode that
//! into `String` directly:
//!
//! ```text
//! ColumnDecode { source: "Rust type `String` (as SQL type `VARCHAR`)
//!                          is not compatible with SQL type `VARBINARY`" }
//! ```
//!
//! Every text column in this file is therefore wrapped in
//! `CAST(... AS CHAR)` (or, where MySQL needs it, `CAST(... AS CHAR
//! CHARACTER SET utf8mb4)`), which forces the server to send a
//! `VARCHAR` and lets sqlx decode straight into `String`. Don't remove
//! the casts even if the column "feels like" a string in the schema.

use anyhow::Result;
use sqlx::{MySql, MySqlPool, Row};

use crate::services::database::types::{
    ColumnDetail, ConstraintInfo, DatabaseInfo, DatabaseSchema, ForeignKeyInfo, IndexInfo,
    QueryExecutionResult, TableInfo, TableSchema,
};

const SYSTEM_SCHEMAS: &[&str] = &["mysql", "information_schema", "performance_schema", "sys"];

pub async fn get_databases(pool: &MySqlPool) -> Result<Vec<DatabaseInfo>> {
    // SHOW DATABASES is the canonical way; cast to CHAR to dodge the
    // VARBINARY decode issue described in the module-level comment.
    let rows = sqlx::query("SELECT CAST(SCHEMA_NAME AS CHAR) AS name FROM information_schema.SCHEMATA")
        .fetch_all(pool)
        .await?;

    let databases = rows
        .into_iter()
        .filter_map(|row| {
            let name: String = row.try_get("name").ok()?;
            if SYSTEM_SCHEMAS.contains(&name.as_str()) {
                None
            } else {
                Some(DatabaseInfo { datname: name })
            }
        })
        .collect();

    Ok(databases)
}

pub async fn get_tables(pool: &MySqlPool) -> Result<Vec<TableInfo>> {
    let query = r#"
        SELECT
            CAST(TABLE_NAME   AS CHAR) AS table_name,
            CAST(TABLE_SCHEMA AS CHAR) AS table_schema,
            CAST(TABLE_TYPE   AS CHAR) AS table_type
        FROM information_schema.TABLES
        WHERE TABLE_SCHEMA = DATABASE()
        ORDER BY TABLE_SCHEMA, TABLE_NAME
    "#;

    let rows = sqlx::query(query).fetch_all(pool).await?;

    Ok(rows
        .into_iter()
        .map(|row| TableInfo {
            table_name: row.try_get("table_name").unwrap_or_default(),
            table_schema: row.try_get("table_schema").unwrap_or_default(),
            table_type: row.try_get("table_type").unwrap_or_default(),
        })
        .collect())
}

pub async fn get_table_columns(
    pool: &MySqlPool,
    table_name: &str,
    table_schema: &str,
) -> QueryExecutionResult {
    let query_str = r#"
        SELECT
            CAST(COLUMN_NAME    AS CHAR) AS column_name,
            CAST(DATA_TYPE      AS CHAR) AS data_type,
            CAST(IS_NULLABLE    AS CHAR) AS is_nullable,
            CAST(COLUMN_DEFAULT AS CHAR) AS column_default,
            ORDINAL_POSITION             AS ordinal_position
        FROM information_schema.COLUMNS
        WHERE TABLE_NAME = ? AND TABLE_SCHEMA = ?
        ORDER BY ORDINAL_POSITION
    "#;

    let query = sqlx::query::<MySql>(query_str)
        .bind(table_name)
        .bind(table_schema);

    super::query::execute_internal(query, pool).await
}

pub async fn get_schema(
    pool: &MySqlPool,
    specific_tables: Option<Vec<String>>,
) -> Result<DatabaseSchema> {
    let table_query = r#"
        SELECT
            CAST(TABLE_NAME    AS CHAR) AS table_name,
            CAST(TABLE_SCHEMA  AS CHAR) AS table_schema,
            CAST(TABLE_TYPE    AS CHAR) AS table_type,
            CAST(TABLE_COMMENT AS CHAR) AS description
        FROM information_schema.TABLES
        WHERE TABLE_SCHEMA = DATABASE()
        ORDER BY TABLE_SCHEMA, TABLE_NAME
    "#;

    let table_rows = sqlx::query(table_query).fetch_all(pool).await?;
    let mut tables = Vec::new();

    for table_row in table_rows {
        let table_name: String = table_row.try_get("table_name").unwrap_or_default();
        let table_schema: String = table_row.try_get("table_schema").unwrap_or_default();
        let table_type: String = table_row.try_get("table_type").unwrap_or_default();
        let description: Option<String> = table_row
            .try_get::<String, _>("description")
            .ok()
            .filter(|s| !s.is_empty());

        if let Some(ref filter_tables) = specific_tables {
            if !filter_tables.contains(&table_name) {
                continue;
            }
        }

        let columns = fetch_table_columns(&table_name, &table_schema, pool).await?;
        let primary_keys = fetch_primary_keys(&table_name, &table_schema, pool).await?;
        let foreign_keys = fetch_foreign_keys(&table_name, &table_schema, pool).await?;
        let indexes = fetch_indexes(&table_name, &table_schema, pool).await?;
        let constraints = fetch_constraints(&table_name, &table_schema, pool).await?;

        tables.push(TableSchema {
            table_name,
            table_schema,
            table_type,
            columns,
            primary_keys,
            foreign_keys,
            indexes,
            constraints,
            description,
        });
    }

    let total_tables = tables.len();
    Ok(DatabaseSchema {
        tables,
        total_tables,
    })
}

async fn fetch_table_columns(
    table_name: &str,
    table_schema: &str,
    pool: &MySqlPool,
) -> Result<Vec<ColumnDetail>> {
    let query = r#"
        SELECT
            CAST(COLUMN_NAME    AS CHAR) AS column_name,
            CAST(DATA_TYPE      AS CHAR) AS data_type,
            CAST(IS_NULLABLE    AS CHAR) AS is_nullable,
            CAST(COLUMN_DEFAULT AS CHAR) AS column_default,
            ORDINAL_POSITION             AS ordinal_position,
            CHARACTER_MAXIMUM_LENGTH     AS character_maximum_length,
            NUMERIC_PRECISION            AS numeric_precision,
            NUMERIC_SCALE                AS numeric_scale,
            CAST(COLUMN_COMMENT AS CHAR) AS description
        FROM information_schema.COLUMNS
        WHERE TABLE_NAME = ? AND TABLE_SCHEMA = ?
        ORDER BY ORDINAL_POSITION
    "#;

    let rows = sqlx::query(query)
        .bind(table_name)
        .bind(table_schema)
        .fetch_all(pool)
        .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let is_nullable: String = row.try_get("is_nullable").unwrap_or_default();
            // information_schema returns these as i64/u64 depending on
            // the column; coerce defensively. NUMERIC_PRECISION in
            // particular is BIGINT UNSIGNED on MySQL 8.4, so try u64
            // first and fall back to i64.
            let character_maximum_length = row
                .try_get::<i64, _>("character_maximum_length")
                .ok()
                .or_else(|| row.try_get::<u64, _>("character_maximum_length").ok().map(|v| v as i64))
                .map(|v| v as i32);
            let numeric_precision = row
                .try_get::<u64, _>("numeric_precision")
                .ok()
                .or_else(|| row.try_get::<i64, _>("numeric_precision").ok().map(|v| v as u64))
                .map(|v| v as i32);
            let numeric_scale = row
                .try_get::<u64, _>("numeric_scale")
                .ok()
                .or_else(|| row.try_get::<i64, _>("numeric_scale").ok().map(|v| v as u64))
                .map(|v| v as i32);
            let ordinal_position = row
                .try_get::<u32, _>("ordinal_position")
                .map(|v| v as i32)
                .or_else(|_| row.try_get::<i64, _>("ordinal_position").map(|v| v as i32))
                .unwrap_or(0);
            let description = row
                .try_get::<String, _>("description")
                .ok()
                .filter(|s| !s.is_empty());

            ColumnDetail {
                column_name: row.try_get("column_name").unwrap_or_default(),
                data_type: row.try_get("data_type").unwrap_or_default(),
                is_nullable: is_nullable == "YES",
                column_default: row.try_get("column_default").ok(),
                ordinal_position,
                character_maximum_length,
                numeric_precision,
                numeric_scale,
                description,
            }
        })
        .collect())
}

async fn fetch_primary_keys(
    table_name: &str,
    table_schema: &str,
    pool: &MySqlPool,
) -> Result<Vec<String>> {
    let query = r#"
        SELECT CAST(COLUMN_NAME AS CHAR) AS column_name
        FROM information_schema.KEY_COLUMN_USAGE
        WHERE CONSTRAINT_NAME = 'PRIMARY'
            AND TABLE_NAME = ?
            AND TABLE_SCHEMA = ?
        ORDER BY ORDINAL_POSITION
    "#;

    let rows = sqlx::query(query)
        .bind(table_name)
        .bind(table_schema)
        .fetch_all(pool)
        .await?;

    Ok(rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("column_name").ok())
        .collect())
}

async fn fetch_foreign_keys(
    table_name: &str,
    table_schema: &str,
    pool: &MySqlPool,
) -> Result<Vec<ForeignKeyInfo>> {
    let query = r#"
        SELECT
            CAST(kcu.CONSTRAINT_NAME         AS CHAR) AS constraint_name,
            CAST(kcu.COLUMN_NAME             AS CHAR) AS column_name,
            CAST(kcu.REFERENCED_TABLE_SCHEMA AS CHAR) AS foreign_table_schema,
            CAST(kcu.REFERENCED_TABLE_NAME   AS CHAR) AS foreign_table_name,
            CAST(kcu.REFERENCED_COLUMN_NAME  AS CHAR) AS foreign_column_name
        FROM information_schema.KEY_COLUMN_USAGE kcu
        WHERE kcu.TABLE_NAME = ?
          AND kcu.TABLE_SCHEMA = ?
          AND kcu.REFERENCED_TABLE_NAME IS NOT NULL
        ORDER BY kcu.CONSTRAINT_NAME, kcu.ORDINAL_POSITION
    "#;

    let rows = sqlx::query(query)
        .bind(table_name)
        .bind(table_schema)
        .fetch_all(pool)
        .await?;

    Ok(rows
        .into_iter()
        .map(|row| ForeignKeyInfo {
            constraint_name: row.try_get("constraint_name").unwrap_or_default(),
            column_name: row.try_get("column_name").unwrap_or_default(),
            foreign_table_schema: row.try_get("foreign_table_schema").unwrap_or_default(),
            foreign_table_name: row.try_get("foreign_table_name").unwrap_or_default(),
            foreign_column_name: row.try_get("foreign_column_name").unwrap_or_default(),
        })
        .collect())
}

async fn fetch_indexes(
    table_name: &str,
    table_schema: &str,
    pool: &MySqlPool,
) -> Result<Vec<IndexInfo>> {
    // GROUP_CONCAT materializes the column list per index; ordered by
    // SEQ_IN_INDEX so multi-column indexes round-trip deterministically.
    let query = r#"
        SELECT
            CAST(INDEX_NAME AS CHAR) AS index_name,
            CAST(GROUP_CONCAT(COLUMN_NAME ORDER BY SEQ_IN_INDEX SEPARATOR ',') AS CHAR) AS columns,
            MAX(NON_UNIQUE) = 0 AS is_unique,
            (INDEX_NAME = 'PRIMARY') AS is_primary,
            CAST(MAX(INDEX_TYPE) AS CHAR) AS index_type
        FROM information_schema.STATISTICS
        WHERE TABLE_NAME = ? AND TABLE_SCHEMA = ?
        GROUP BY INDEX_NAME
    "#;

    let rows = sqlx::query(query)
        .bind(table_name)
        .bind(table_schema)
        .fetch_all(pool)
        .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let columns_csv: String = row.try_get("columns").unwrap_or_default();
            let columns = if columns_csv.is_empty() {
                Vec::new()
            } else {
                columns_csv.split(',').map(|s| s.to_string()).collect()
            };
            // is_unique / is_primary are computed booleans; MySQL
            // returns these as i64 (0/1).
            let is_unique = row.try_get::<i64, _>("is_unique").unwrap_or(0) != 0;
            let is_primary = row.try_get::<i64, _>("is_primary").unwrap_or(0) != 0;
            IndexInfo {
                index_name: row.try_get("index_name").unwrap_or_default(),
                columns,
                is_unique,
                is_primary,
                index_type: row.try_get("index_type").unwrap_or_default(),
            }
        })
        .collect())
}

async fn fetch_constraints(
    table_name: &str,
    table_schema: &str,
    pool: &MySqlPool,
) -> Result<Vec<ConstraintInfo>> {
    // UNIQUE constraints + CHECK constraints (CHECK requires MySQL 8.0+).
    // Synthesize the column list from KEY_COLUMN_USAGE for UNIQUEs and
    // pull check_clause from CHECK_CONSTRAINTS for CHECKs.
    let query = r#"
        SELECT
            CAST(tc.CONSTRAINT_NAME AS CHAR) AS constraint_name,
            CAST(tc.CONSTRAINT_TYPE AS CHAR) AS constraint_type,
            CAST((
                SELECT GROUP_CONCAT(kcu.COLUMN_NAME ORDER BY kcu.ORDINAL_POSITION SEPARATOR ',')
                FROM information_schema.KEY_COLUMN_USAGE kcu
                WHERE kcu.CONSTRAINT_NAME = tc.CONSTRAINT_NAME
                  AND kcu.TABLE_NAME = tc.TABLE_NAME
                  AND kcu.TABLE_SCHEMA = tc.TABLE_SCHEMA
            ) AS CHAR) AS columns,
            CAST((
                SELECT cc.CHECK_CLAUSE
                FROM information_schema.CHECK_CONSTRAINTS cc
                WHERE cc.CONSTRAINT_NAME = tc.CONSTRAINT_NAME
                  AND cc.CONSTRAINT_SCHEMA = tc.TABLE_SCHEMA
                LIMIT 1
            ) AS CHAR) AS check_clause
        FROM information_schema.TABLE_CONSTRAINTS tc
        WHERE tc.TABLE_NAME = ?
          AND tc.TABLE_SCHEMA = ?
          AND tc.CONSTRAINT_TYPE IN ('UNIQUE', 'CHECK')
    "#;

    let rows = sqlx::query(query)
        .bind(table_name)
        .bind(table_schema)
        .fetch_all(pool)
        .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let columns_csv: Option<String> = row.try_get("columns").ok();
            let columns = match columns_csv {
                Some(csv) if !csv.is_empty() => csv.split(',').map(|s| s.to_string()).collect(),
                _ => Vec::new(),
            };
            ConstraintInfo {
                constraint_name: row.try_get("constraint_name").unwrap_or_default(),
                constraint_type: row.try_get("constraint_type").unwrap_or_default(),
                columns,
                check_clause: row.try_get("check_clause").ok(),
            }
        })
        .collect())
}
