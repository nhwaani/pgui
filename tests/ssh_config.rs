//! Tests for `services::ssh::config` (SshAuth + SshConfig).
//!
//! No I/O — pure type / serde checks. The actual SSH tunnel
//! (`SshTunnel`) needs a live SSH server and is exercised manually.

use pgui::services::ssh::{SshAuth, SshConfig};

#[test]
fn default_port_is_22() {
    assert_eq!(SshConfig::default().port, 22);
}

#[test]
fn default_auth_is_agent() {
    assert!(matches!(SshConfig::default().auth, SshAuth::Agent));
}

#[test]
fn ssh_auth_partial_eq() {
    // Two KeyFile values with the same path are equal; differing paths
    // are not. The connection form relies on PartialEq when deciding
    // whether to re-prompt for a passphrase.
    let a = SshAuth::KeyFile {
        path: "/a".to_string(),
    };
    let b = SshAuth::KeyFile {
        path: "/a".to_string(),
    };
    let c = SshAuth::KeyFile {
        path: "/b".to_string(),
    };
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_ne!(a, SshAuth::Agent);
}

#[test]
fn ssh_auth_serde_tagging() {
    let json = serde_json::to_string(&SshAuth::Agent).unwrap();
    assert_eq!(json, r#"{"type":"agent"}"#);
    let json = serde_json::to_string(&SshAuth::KeyFile {
        path: "/x".to_string(),
    })
    .unwrap();
    assert_eq!(json, r#"{"type":"key_file","path":"/x"}"#);
}

#[test]
fn ssh_config_serde_keyfile_roundtrip() {
    let cfg = SshConfig {
        host: "bastion.example.com".to_string(),
        port: 2222,
        username: "deploy".to_string(),
        auth: SshAuth::KeyFile {
            path: "/Users/me/.ssh/id_ed25519".to_string(),
        },
    };
    let json = serde_json::to_string(&cfg).unwrap();
    assert!(json.contains("\"type\":\"key_file\""), "got {}", json);
    let back: SshConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn ssh_config_serde_agent_roundtrip() {
    let cfg = SshConfig {
        host: "h".to_string(),
        port: 22,
        username: "u".to_string(),
        auth: SshAuth::Agent,
    };
    let json = serde_json::to_string(&cfg).unwrap();
    assert!(json.contains("\"type\":\"agent\""), "got {}", json);
    let back: SshConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(cfg, back);
}
