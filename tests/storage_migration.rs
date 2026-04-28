//! Integration tests for `services::storage`.
//!
//! These touch a temp-file SQLite database and the (in-memory) keyring
//! provided by `tests/common/mod.rs`. They cover:
//!
//! - Schema initialization on a fresh file.
//! - Migration from a pre-MySQL/SSH legacy schema (only the original 7
//!   columns + a populated row), including idempotency on re-run.
//! - Round-tripping connections through `ConnectionsRepository`:
//!   Postgres no SSH, MySQL + SSH key-file, MySQL + SSH agent.
//! - CRUD edges: duplicate-name rejection, update flips, delete
//!   removes both row and keyring entry, case-sensitive `exists_by_name`.
//! - SSH key passphrase keyring helpers.
//!
//! Live database connections (PG, MySQL) and the SSH tunnel itself are
//! intentionally **not** covered here; those need Docker / an SSH
//! server and live in the manual smoke-test path documented in the
//! README.

mod common;

use std::str::FromStr;

use pgui::services::ssh::{SshAuth, SshConfig};
use pgui::services::storage::{
    AppStore, ConnectionInfo, ConnectionsRepository, DatabaseDriver, SslMode,
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use uuid::Uuid;

use common::{fresh_store, init_keyring_mock};

/// Build a SQLite pool against `path` without going through `AppStore`,
/// so we can simulate older databases that lack the new columns.
async fn raw_pool(path: &std::path::Path) -> SqlitePool {
    let opts = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))
        .unwrap()
        .create_if_missing(true);
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(opts)
        .await
        .unwrap()
}

#[test]
fn fresh_database_has_all_columns() {
    smol::block_on(async {
        let (_dir, store) = fresh_store().await;
        // The repository's load_all SELECTs every column we expect; if
        // any is missing the call errors and the test fails.
        assert!(store.connections().load_all().await.unwrap().is_empty());
    });
}

#[test]
fn migration_from_legacy_schema_adds_all_columns() {
    smol::block_on(async {
        init_keyring_mock();
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("legacy.db");

        // 1. Create a legacy-shaped table (the schema as it was before
        //    the MySQL+SSH PR), populate one row, then close the pool.
        {
            let pool = raw_pool(&db_path).await;
            sqlx::query(
                r#"
                CREATE TABLE connections (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL UNIQUE,
                    hostname TEXT NOT NULL,
                    username TEXT NOT NULL,
                    database TEXT NOT NULL,
                    port INTEGER NOT NULL,
                    ssl_mode TEXT NOT NULL DEFAULT 'prefer'
                )
                "#,
            )
            .execute(&pool)
            .await
            .unwrap();

            sqlx::query(
                r#"INSERT INTO connections
                   (id, name, hostname, username, database, port, ssl_mode)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
            )
            .bind("00000000-0000-0000-0000-000000000001")
            .bind("legacy-pg")
            .bind("db.example.com")
            .bind("alice")
            .bind("appdb")
            .bind(5432_i64)
            .bind("prefer")
            .execute(&pool)
            .await
            .unwrap();
            pool.close().await;
        }

        // 2. Open via AppStore — initialize_schema is a no-op (table
        //    exists); migrate_schema must add the new columns.
        let store = AppStore::from_path(db_path).await.unwrap();

        // 3. The legacy row is loadable, defaults are filled in.
        let conns = store.connections().load_all().await.unwrap();
        assert_eq!(conns.len(), 1);
        let c = &conns[0];
        assert_eq!(c.name, "legacy-pg");
        assert_eq!(c.driver, DatabaseDriver::Postgres, "driver default");
        assert!(c.ssh.is_none(), "legacy row should have no SSH");
        assert_eq!(c.port, 5432);
    });
}

#[test]
fn migration_is_idempotent() {
    smol::block_on(async {
        let (dir, store1) = fresh_store().await;
        let path = dir.path().join("pgui.db");
        // Drop store1's pool and reopen — migrate_schema runs again.
        drop(store1);
        let store2 = AppStore::from_path(path.clone()).await.unwrap();
        drop(store2);
        let store3 = AppStore::from_path(path).await.unwrap();

        assert!(store3.connections().load_all().await.unwrap().is_empty());
    });
}

#[test]
fn create_load_postgres_no_ssh_roundtrip() {
    smol::block_on(async {
        let (_dir, store) = fresh_store().await;
        let repo = store.connections();

        let info = ConnectionInfo {
            id: Uuid::new_v4(),
            name: "pg-direct".to_string(),
            driver: DatabaseDriver::Postgres,
            hostname: "localhost".to_string(),
            username: "alice".to_string(),
            password: "supersecret".to_string(),
            database: "appdb".to_string(),
            port: 5432,
            ssl_mode: SslMode::Require,
            ssh: None,
        };
        repo.create(&info).await.unwrap();

        let loaded = repo.load_all().await.unwrap();
        assert_eq!(loaded.len(), 1);
        let l = &loaded[0];
        assert_eq!(l.id, info.id);
        assert_eq!(l.name, info.name);
        assert_eq!(l.driver, DatabaseDriver::Postgres);
        assert_eq!(l.port, 5432);
        assert_eq!(l.ssl_mode, SslMode::Require);
        assert!(l.ssh.is_none());
        assert_eq!(l.password, "", "password loaded on-demand, not eagerly");

        // The keyring (in-memory) does have the password.
        let pw = ConnectionsRepository::get_connection_password(&info.id).unwrap();
        assert_eq!(pw, "supersecret");
    });
}

#[test]
fn create_load_mysql_with_ssh_keyfile_roundtrip() {
    smol::block_on(async {
        let (_dir, store) = fresh_store().await;
        let repo = store.connections();

        let info = ConnectionInfo {
            id: Uuid::new_v4(),
            name: "mysql-via-bastion".to_string(),
            driver: DatabaseDriver::MySql,
            hostname: "10.0.0.42".to_string(),
            username: "app".to_string(),
            password: "app-pass".to_string(),
            database: "appdb".to_string(),
            port: 3306,
            ssl_mode: SslMode::Prefer,
            ssh: Some(SshConfig {
                host: "bastion.internal".to_string(),
                port: 2222,
                username: "deploy".to_string(),
                auth: SshAuth::KeyFile {
                    path: "/Users/me/.ssh/id_ed25519".to_string(),
                },
            }),
        };
        repo.create(&info).await.unwrap();

        let loaded = repo.load_all().await.unwrap();
        assert_eq!(loaded.len(), 1);
        let l = &loaded[0];
        assert_eq!(l.driver, DatabaseDriver::MySql);
        assert_eq!(l.port, 3306);
        let ssh = l.ssh.as_ref().expect("ssh should be present");
        assert_eq!(ssh.host, "bastion.internal");
        assert_eq!(ssh.port, 2222);
        assert_eq!(ssh.username, "deploy");
        match &ssh.auth {
            SshAuth::KeyFile { path } => assert_eq!(path, "/Users/me/.ssh/id_ed25519"),
            other => panic!("unexpected auth: {:?}", other),
        }
    });
}

#[test]
fn create_load_mysql_with_ssh_agent() {
    smol::block_on(async {
        let (_dir, store) = fresh_store().await;
        let repo = store.connections();

        let info = ConnectionInfo {
            id: Uuid::new_v4(),
            name: "mysql-agent".to_string(),
            driver: DatabaseDriver::MySql,
            hostname: "db.private".to_string(),
            username: "ro".to_string(),
            password: "ro-pass".to_string(),
            database: "metrics".to_string(),
            port: 3306,
            ssl_mode: SslMode::Disable,
            ssh: Some(SshConfig {
                host: "jump.example.com".to_string(),
                port: 22,
                username: "ops".to_string(),
                auth: SshAuth::Agent,
            }),
        };
        repo.create(&info).await.unwrap();

        let loaded = &repo.load_all().await.unwrap()[0];
        let ssh = loaded.ssh.as_ref().unwrap();
        assert!(matches!(ssh.auth, SshAuth::Agent));
    });
}

#[test]
fn duplicate_name_is_rejected_on_create() {
    smol::block_on(async {
        let (_dir, store) = fresh_store().await;
        let repo = store.connections();

        let mut a = ConnectionInfo::default();
        a.id = Uuid::new_v4();
        a.name = "dup".to_string();
        repo.create(&a).await.unwrap();

        let mut b = ConnectionInfo::default();
        b.id = Uuid::new_v4();
        b.name = "dup".to_string();
        let err = repo.create(&b).await.unwrap_err();
        assert!(
            err.to_string().contains("already exists"),
            "expected already-exists error, got: {}",
            err
        );
    });
}

#[test]
fn update_changes_driver_and_ssh_fields() {
    smol::block_on(async {
        let (_dir, store) = fresh_store().await;
        let repo = store.connections();

        let id = Uuid::new_v4();
        let mut info = ConnectionInfo {
            id,
            name: "evolves".to_string(),
            driver: DatabaseDriver::Postgres,
            hostname: "h".to_string(),
            username: "u".to_string(),
            password: "p".to_string(),
            database: "d".to_string(),
            port: 5432,
            ssl_mode: SslMode::Prefer,
            ssh: None,
        };
        repo.create(&info).await.unwrap();

        // Switch to MySQL + add an SSH agent tunnel.
        info.driver = DatabaseDriver::MySql;
        info.port = 3306;
        info.ssh = Some(SshConfig {
            host: "ssh.example".to_string(),
            port: 22,
            username: "me".to_string(),
            auth: SshAuth::Agent,
        });
        repo.update(&info).await.unwrap();

        let loaded = repo.load_all().await.unwrap();
        let l = &loaded[0];
        assert_eq!(l.driver, DatabaseDriver::MySql);
        assert_eq!(l.port, 3306);
        let ssh = l.ssh.as_ref().unwrap();
        assert_eq!(ssh.host, "ssh.example");
        assert!(matches!(ssh.auth, SshAuth::Agent));

        // Drop SSH back to None and verify the row reflects that.
        info.ssh = None;
        repo.update(&info).await.unwrap();
        let l2 = &repo.load_all().await.unwrap()[0];
        assert!(l2.ssh.is_none());
    });
}

#[test]
fn delete_removes_row_and_password() {
    smol::block_on(async {
        let (_dir, store) = fresh_store().await;
        let repo = store.connections();

        let id = Uuid::new_v4();
        let mut info = ConnectionInfo::default();
        info.id = id;
        info.name = "to-be-deleted".to_string();
        info.password = "ephemeral".to_string();
        repo.create(&info).await.unwrap();

        assert_eq!(
            ConnectionsRepository::get_connection_password(&id).unwrap(),
            "ephemeral"
        );

        repo.delete(&id).await.unwrap();

        assert!(repo.load_all().await.unwrap().is_empty());
        assert!(ConnectionsRepository::get_connection_password(&id).is_err());
    });
}

#[test]
fn ssh_key_passphrase_roundtrip_via_keyring() {
    init_keyring_mock();
    let id = Uuid::new_v4();

    assert!(ConnectionsRepository::get_ssh_key_passphrase(&id).is_none());

    ConnectionsRepository::store_ssh_key_passphrase(&id, "hunter2").unwrap();
    assert_eq!(
        ConnectionsRepository::get_ssh_key_passphrase(&id).as_deref(),
        Some("hunter2")
    );

    // Empty string clears it.
    ConnectionsRepository::store_ssh_key_passphrase(&id, "").unwrap();
    assert!(ConnectionsRepository::get_ssh_key_passphrase(&id).is_none());
}

#[test]
fn exists_by_name_is_case_sensitive_and_scoped() {
    smol::block_on(async {
        let (_dir, store) = fresh_store().await;
        let repo = store.connections();

        let mut info = ConnectionInfo::default();
        info.id = Uuid::new_v4();
        info.name = "Prod".to_string();
        repo.create(&info).await.unwrap();

        assert!(repo.exists_by_name("Prod").await.unwrap());
        // Case-sensitive: SQLite's default `=` on TEXT is binary-collation.
        assert!(!repo.exists_by_name("prod").await.unwrap());
        assert!(!repo.exists_by_name("Staging").await.unwrap());
    });
}
