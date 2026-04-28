//! Type / serde / connect-options tests for `services::storage::types`.
//!
//! These exercise pure data structures and their conversions to sqlx's
//! `PgConnectOptions` / `MySqlConnectOptions`. No SQLite, no keyring,
//! no network.

use gpui_component::select::SelectItem;
use pgui::services::ssh::{SshAuth, SshConfig};
use pgui::services::storage::{ConnectionInfo, DatabaseDriver, SslMode};

// ============================================================================
// DatabaseDriver
// ============================================================================

#[test]
fn database_driver_db_str_roundtrip() {
    for d in DatabaseDriver::all() {
        assert_eq!(DatabaseDriver::from_db_str(d.to_db_str()), d);
    }
    // Unknown values fall back to Postgres for forward-compat.
    assert_eq!(
        DatabaseDriver::from_db_str("future-driver"),
        DatabaseDriver::Postgres
    );
    // Empty string -> Postgres (avoids panicking on legacy rows).
    assert_eq!(DatabaseDriver::from_db_str(""), DatabaseDriver::Postgres);
}

#[test]
fn database_driver_index_roundtrip() {
    for d in DatabaseDriver::all() {
        assert_eq!(DatabaseDriver::from_index(d.to_index()), d);
    }
    // Out-of-range -> Postgres.
    assert_eq!(DatabaseDriver::from_index(99), DatabaseDriver::Postgres);
}

#[test]
fn database_driver_default_ports() {
    assert_eq!(DatabaseDriver::Postgres.default_port(), 5432);
    assert_eq!(DatabaseDriver::MySql.default_port(), 3306);
}

#[test]
fn database_driver_serde_roundtrip() {
    for d in DatabaseDriver::all() {
        let json = serde_json::to_string(&d).unwrap();
        let back: DatabaseDriver = serde_json::from_str(&json).unwrap();
        assert_eq!(d, back);
        assert!(
            json == "\"postgres\"" || json == "\"mysql\"",
            "unexpected driver json {}",
            json
        );
    }
}

#[test]
fn database_driver_default_is_postgres() {
    assert_eq!(DatabaseDriver::default(), DatabaseDriver::Postgres);
}

#[test]
fn database_driver_select_item_titles() {
    assert_eq!(
        DatabaseDriver::Postgres.title().to_string(),
        "PostgreSQL"
    );
    assert_eq!(DatabaseDriver::MySql.title().to_string(), "MySQL");
}

// ============================================================================
// SslMode
// ============================================================================

#[test]
fn ssl_mode_db_str_roundtrip() {
    for m in SslMode::all() {
        assert_eq!(SslMode::from_db_str(m.to_db_str()), m);
    }
    assert_eq!(SslMode::from_db_str("banana"), SslMode::Prefer);
}

#[test]
fn ssl_mode_index_roundtrip() {
    for m in SslMode::all() {
        assert_eq!(SslMode::from_index(m.to_index()), m);
    }
    assert_eq!(SslMode::from_index(42), SslMode::Prefer);
}

#[test]
fn ssl_mode_pg_mappings_complete() {
    // sqlx's SslMode types don't impl PartialEq, so compare via Debug.
    let cases = [
        (SslMode::Disable, "Disable"),
        (SslMode::Prefer, "Prefer"),
        (SslMode::Require, "Require"),
        (SslMode::VerifyCa, "VerifyCa"),
        (SslMode::VerifyFull, "VerifyFull"),
    ];
    for (m, expected) in cases {
        assert_eq!(format!("{:?}", m.to_pg_ssl_mode()), expected);
    }
}

#[test]
fn ssl_mode_mysql_mappings_complete() {
    let cases = [
        (SslMode::Disable, "Disabled"),
        (SslMode::Prefer, "Preferred"),
        (SslMode::Require, "Required"),
        (SslMode::VerifyCa, "VerifyCa"),
        (SslMode::VerifyFull, "VerifyIdentity"),
    ];
    for (m, expected) in cases {
        assert_eq!(format!("{:?}", m.to_mysql_ssl_mode()), expected);
    }
}

// ============================================================================
// ConnectionInfo
// ============================================================================

#[test]
fn connection_info_empty_password_is_skipped() {
    let mut info = ConnectionInfo::default();
    info.password = String::new();
    let json = serde_json::to_string(&info).unwrap();
    assert!(
        !json.contains("\"password\""),
        "unexpected password key: {}",
        json
    );
}

#[test]
fn connection_info_default_has_no_ssh() {
    let info = ConnectionInfo::default();
    assert!(info.ssh.is_none());
    assert_eq!(info.driver, DatabaseDriver::Postgres);
}

#[test]
fn connection_info_ssh_skipped_when_none() {
    let info = ConnectionInfo::default();
    let json = serde_json::to_string(&info).unwrap();
    assert!(
        !json.contains("\"ssh\""),
        "ssh should be skip_serializing_if=None: {}",
        json
    );
}

#[test]
fn connection_info_with_ssh_serde_roundtrip() {
    let mut info = ConnectionInfo::default();
    info.password = String::new();
    info.driver = DatabaseDriver::MySql;
    info.port = 3306;
    info.ssh = Some(SshConfig {
        host: "jump.example.com".to_string(),
        port: 22,
        username: "ops".to_string(),
        auth: SshAuth::KeyFile {
            path: "/home/ops/.ssh/id_rsa".to_string(),
        },
    });
    let json = serde_json::to_string(&info).unwrap();
    let back: ConnectionInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(back.driver, DatabaseDriver::MySql);
    assert_eq!(back.port, 3306);
    assert_eq!(back.ssh, info.ssh);
}

#[test]
fn connection_info_legacy_json_without_driver_or_ssh() {
    // Pre-MySQL/SSH saves should still deserialize with serde defaults
    // filling in the new fields.
    let json = r#"{
        "id": "00000000-0000-0000-0000-000000000001",
        "name": "old",
        "hostname": "db",
        "username": "u",
        "database": "d",
        "port": 5432
    }"#;
    let info: ConnectionInfo = serde_json::from_str(json).unwrap();
    assert_eq!(info.driver, DatabaseDriver::Postgres);
    assert!(info.ssh.is_none());
    assert_eq!(info.ssl_mode, SslMode::Prefer);
}

#[test]
fn pg_connect_options_use_overridden_host_port() {
    // When a tunnel is in use we connect via 127.0.0.1:<random>; make
    // sure the override knobs actually substitute that endpoint.
    let mut info = ConnectionInfo::default();
    info.hostname = "db.internal".to_string();
    info.port = 5432;
    let opts = info.to_pg_connect_options_for("127.0.0.1", 49152);
    assert_eq!(opts.get_host(), "127.0.0.1");
    assert_eq!(opts.get_port(), 49152);
}

#[test]
fn mysql_connect_options_use_overridden_host_port() {
    let mut info = ConnectionInfo::default();
    info.driver = DatabaseDriver::MySql;
    info.hostname = "mysql.internal".to_string();
    info.port = 3306;
    info.username = "app".to_string();
    info.database = "appdb".to_string();
    let opts = info.to_mysql_connect_options_for("127.0.0.1", 50001);
    assert_eq!(opts.get_host(), "127.0.0.1");
    assert_eq!(opts.get_port(), 50001);
}

#[test]
fn pg_connect_options_carry_credentials_and_database() {
    let mut info = ConnectionInfo::default();
    info.username = "alice".to_string();
    info.database = "appdb".to_string();
    info.password = "secret".to_string();
    let opts = info.to_pg_connect_options_for("db", 5432);
    assert_eq!(opts.get_username(), "alice");
    assert_eq!(opts.get_database(), Some("appdb"));
}

#[test]
fn empty_database_falls_back_to_server_default_pg() {
    // When the form leaves Database blank we must NOT call .database("")
    // on sqlx's PgConnectOptions — that would tell PG to connect to a
    // database literally named empty-string. Instead we leave it unset
    // so PG uses its default (a database named after the user).
    let mut info = ConnectionInfo::default();
    info.database = String::new();
    let opts = info.to_pg_connect_options_for("db", 5432);
    assert_eq!(opts.get_database(), None);
}

#[test]
fn empty_database_falls_back_to_server_default_mysql() {
    let mut info = ConnectionInfo::default();
    info.driver = DatabaseDriver::MySql;
    info.database = String::new();
    let opts = info.to_mysql_connect_options_for("db", 3306);
    assert_eq!(opts.get_database(), None);
}
