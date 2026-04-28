// Smoke-test the live MySQL via the pgui lib — exercises the same code
// paths that connect_async hits after a successful pool open.

use pgui::services::storage::{ConnectionInfo, DatabaseDriver, SslMode};
use pgui::services::DatabaseManager;
use uuid::Uuid;

fn main() {
    smol::block_on(async {
        let info = ConnectionInfo {
            id: Uuid::new_v4(),
            name: "smoke".into(),
            driver: DatabaseDriver::MySql,
            hostname: "127.0.0.1".into(),
            username: "test".into(),
            password: "test".into(),
            database: "test".into(),
            port: 3306,
            ssl_mode: SslMode::Disable,
            ssh: None,
        };

        let mgr = DatabaseManager::new();
        println!("connecting...");
        match mgr.connect(&info).await {
            Ok(_) => println!("OK connect"),
            Err(e) => { eprintln!("connect error: {e}"); return; }
        }

        println!("get_tables...");
        match mgr.get_tables().await {
            Ok(v) => println!("  {} tables", v.len()),
            Err(e) => eprintln!("  tables error: {e}"),
        }

        println!("get_databases...");
        match mgr.get_databases().await {
            Ok(v) => println!("  {} dbs", v.len()),
            Err(e) => eprintln!("  dbs error: {e}"),
        }

        println!("get_schema...");
        match mgr.get_schema(None).await {
            Ok(s) => println!("  {} tables in schema", s.total_tables),
            Err(e) => eprintln!("  schema error: {e}"),
        }

        // Stress the row decoder against the diverse-types table.
        for sql in [
            "SELECT id, name, email FROM users ORDER BY id LIMIT 3",
            "SELECT id, enum_val, set_val, json_val FROM mysql_types_test",
            "SHOW TABLES",
            "DESCRIBE products",
        ] {
            println!("\n=== {sql}");
            match mgr.execute_query_enhanced(sql).await {
                pgui::services::QueryExecutionResult::Select(r) => {
                    println!("  {} cols, {} rows in {}ms", r.columns.len(), r.row_count, r.execution_time_ms);
                    for col in &r.columns {
                        print!("  {:>14}", col.name);
                    }
                    println!();
                    for row in r.rows.iter().take(5) {
                        for cell in &row.cells {
                            let v = if cell.value.len() > 14 { &cell.value[..14] } else { &cell.value };
                            print!("  {:>14}", v);
                        }
                        println!();
                    }
                }
                pgui::services::QueryExecutionResult::Modified(m) => {
                    println!("  modified: {} rows in {}ms", m.rows_affected, m.execution_time_ms);
                }
                pgui::services::QueryExecutionResult::Error(e) => {
                    eprintln!("  error: {} ({}ms)", e.message, e.execution_time_ms);
                }
            }
        }

        println!("\ndone");
    });
}
