//! Helpers shared between integration tests.
//!
//! Each `tests/<file>.rs` is compiled as its own integration-test
//! binary. To share helpers across them, Cargo treats anything under
//! `tests/common/` as a regular Rust submodule and *not* as a separate
//! test binary. Files in here are pulled into each test crate via
//! `mod common;`.
//!
//! ## What lives here
//!
//! - `init_keyring_mock` and the in-memory keyring backend.
//! - Tempfile + AppStore wiring for tests that touch SQLite.

#![allow(dead_code)]

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Mutex, Once, OnceLock};

use keyring::credential::{
    Credential, CredentialApi, CredentialBuilder, CredentialBuilderApi, CredentialPersistence,
};
use pgui::services::storage::AppStore;
use tempfile::TempDir;

// ============================================================================
// In-memory keyring backend
// ============================================================================
//
// `keyring`'s bundled `mock` builder produces a *fresh* `MockCredential`
// on every `Entry::new()` call, so a value written through one Entry
// can't be read through another — which is the exact pattern this
// codebase uses (write at create-time, read later via a separate
// Entry). We bring our own tiny in-memory builder backed by a
// process-wide `HashMap`, modeling the real-store contract: any two
// entries with the same `(service, user)` see the same secret.

fn store() -> &'static Mutex<HashMap<(String, String), Vec<u8>>> {
    static STORE: OnceLock<Mutex<HashMap<(String, String), Vec<u8>>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

struct InMemoryCredential {
    service: String,
    user: String,
}

impl CredentialApi for InMemoryCredential {
    fn set_secret(&self, secret: &[u8]) -> keyring::Result<()> {
        store()
            .lock()
            .unwrap()
            .insert((self.service.clone(), self.user.clone()), secret.to_vec());
        Ok(())
    }

    fn get_secret(&self) -> keyring::Result<Vec<u8>> {
        store()
            .lock()
            .unwrap()
            .get(&(self.service.clone(), self.user.clone()))
            .cloned()
            .ok_or(keyring::Error::NoEntry)
    }

    fn delete_credential(&self) -> keyring::Result<()> {
        let removed = store()
            .lock()
            .unwrap()
            .remove(&(self.service.clone(), self.user.clone()));
        match removed {
            Some(_) => Ok(()),
            None => Err(keyring::Error::NoEntry),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct InMemoryBuilder;

impl CredentialBuilderApi for InMemoryBuilder {
    fn build(
        &self,
        _target: Option<&str>,
        service: &str,
        user: &str,
    ) -> keyring::Result<Box<Credential>> {
        Ok(Box::new(InMemoryCredential {
            service: service.to_string(),
            user: user.to_string(),
        }))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn persistence(&self) -> CredentialPersistence {
        CredentialPersistence::ProcessOnly
    }
}

static KEYRING_INIT: Once = Once::new();

/// Install the in-memory keyring backend once per process. Safe to
/// call repeatedly. Each integration-test binary is its own process,
/// so the in-memory store does *not* leak between test files.
pub fn init_keyring_mock() {
    KEYRING_INIT.call_once(|| {
        keyring::set_default_credential_builder(
            Box::new(InMemoryBuilder) as Box<CredentialBuilder>,
        );
    });
}

// ============================================================================
// AppStore fixtures
// ============================================================================

/// Create a fresh `AppStore` against a temp-file SQLite database.
///
/// Returns `(temp_dir, store)`. The caller must hold `temp_dir` for the
/// lifetime of the test so the underlying file isn't dropped early.
pub async fn fresh_store() -> (TempDir, AppStore) {
    init_keyring_mock();
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("pgui.db");
    let store = AppStore::from_path(db_path)
        .await
        .expect("AppStore::from_path");
    (dir, store)
}
