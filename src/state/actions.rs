//! Actions that orchestrate state changes across multiple global states.
//!
//! These functions handle cross-cutting concerns like connecting/disconnecting
//! from databases, which need to update multiple states simultaneously.

use std::time::Duration;

use gpui::*;
use gpui_component::WindowExt as _;
use gpui_component::notification::NotificationType;

use crate::services::{AppStore, ConnectionInfo, ConnectionsRepository, DatabaseManager};

use super::connection::{ConnectionState, ConnectionStatus};
use super::database::DatabaseState;
use super::editor::EditorState;

// =============================================================================
// Connection Lifecycle
// =============================================================================

/// Initiates a connection to the database.
/// Updates ConnectionState, EditorState, and DatabaseState on success.
pub fn connect(connection_info: &ConnectionInfo, cx: &mut App) {
    cx.update_global::<ConnectionState, _>(|state, _cx| {
        state.connection_state = ConnectionStatus::Connecting;
    });

    let cic = connection_info.clone();
    let db_manager = cx.global::<ConnectionState>().db_manager.clone();

    cx.spawn(async move |cx| connect_async(cic, db_manager, cx).await)
        .detach();
}

/// Disconnects from the current database.
/// Updates ConnectionState and LLMState.
pub fn disconnect(cx: &mut App) {
    let db_manager = cx.global::<ConnectionState>().db_manager.clone();
    cx.spawn(async move |cx| disconnect_async(db_manager, cx).await)
        .detach();
}

/// Changes to a different database on the same server.
/// Disconnects from current database and reconnects to the new one.
pub fn change_database(database_name: String, cx: &mut App) {
    let current_connection = cx.global::<ConnectionState>().active_connection.clone();

    if let Some(mut new_connection) = current_connection {
        new_connection.database = database_name;

        let db_manager = cx.global::<ConnectionState>().db_manager.clone();
        cx.spawn(async move |cx| {
            disconnect_async(db_manager.clone(), cx).await;
            // Wait a brief moment for cleanup
            cx.background_executor()
                .timer(Duration::from_millis(100))
                .await;
            // Connect to the new database
            connect_async(new_connection, db_manager, cx).await;
        })
        .detach();
    }
}

// =============================================================================
// Connection CRUD Operations
// =============================================================================

/// Adds a new connection to the saved connections store.
pub fn add_connection(connection: ConnectionInfo, cx: &mut App) {
    cx.spawn(async move |cx| {
        if let Ok(store) = AppStore::singleton().await {
            if let Ok(_) = store.connections().create(&connection).await {
                if let Ok(connections) = store.connections().load_all().await {
                    let _ = cx.update_global::<ConnectionState, _>(|app_state, _cx| {
                        app_state.saved_connections = connections;
                        app_state.active_connection = None;
                    });
                }
            }
        }
    })
    .detach();
}

/// Updates an existing connection in the saved connections store.
pub fn update_connection(connection: ConnectionInfo, cx: &mut App) {
    cx.spawn(async move |cx| {
        if let Ok(store) = AppStore::singleton().await {
            if let Ok(_) = store.connections().update(&connection).await {
                if let Ok(connections) = store.connections().load_all().await {
                    let _ = cx.update_global::<ConnectionState, _>(|app_state, _cx| {
                        app_state.saved_connections = connections;
                        app_state.active_connection = Some(connection);
                    });
                }
            }
        }
    })
    .detach();
}

/// Deletes a connection from the saved connections store.
pub fn delete_connection(connection: ConnectionInfo, cx: &mut App) {
    let conn = connection.clone();
    cx.spawn(async move |cx| {
        if let Ok(store) = AppStore::singleton().await {
            if let Ok(_) = store.connections().delete(&conn.id).await {
                if let Ok(connections) = store.connections().load_all().await {
                    let _ = cx.update_global::<ConnectionState, _>(|app_state, _cx| {
                        app_state.saved_connections = connections;
                    });
                }
            }
        }
    })
    .detach();
}

// =============================================================================
// Private Async Helpers
// =============================================================================

async fn connect_async(mut cic: ConnectionInfo, db_manager: DatabaseManager, cx: &mut AsyncApp) {
    // The form may have already supplied a password (when the user just
    // typed one and clicked Connect). Only consult the keychain when the
    // ConnectionInfo arrives without a password — e.g. when the user
    // clicks Connect on a saved entry from the connection list, where
    // load_all() deliberately leaves the password empty.
    if cic.password.is_empty() {
        match ConnectionsRepository::get_connection_password(&cic.id) {
            Ok(password) => cic.password = password,
            Err(_) => {
                notify_connect_failure(
                    cx,
                    "No password on file for this connection. Type one in the form and \
                     click Connect (or Update to persist it).",
                );
                let _ = cx.update_global::<ConnectionState, _>(|state, _cx| {
                    state.connection_state = ConnectionStatus::Disconnected;
                });
                return;
            }
        }
    }

    let connect_result = db_manager.connect(&cic).await;
    if let Ok(_) = connect_result {
        if let Ok(tables) = db_manager.get_tables().await {
            let _ = cx.update_global::<EditorState, _>(|state, _cx| {
                state.tables = tables;
            });
        }

        if let Ok(schema) = db_manager.get_schema(None).await {
            let _ = cx.update_global::<EditorState, _>(|state, _cx| {
                state.schema = Some(schema);
            });
        }

        if let Ok(databases) = db_manager.get_databases().await {
            let _ = cx.update_global::<DatabaseState, _>(|state, _cx| {
                state.databases = databases;
            });
        }

        let _ = cx.update_global::<ConnectionState, _>(|state, _cx| {
            state.active_connection = Some(cic);
            state.connection_state = ConnectionStatus::Connected;
        });

        // Connection monitoring loop
        loop {
            let mut connected = db_manager.is_connected().await;
            if !connected {
                let _ = cx.update_global::<ConnectionState, _>(|state, _cx| {
                    state.active_connection = None;
                    state.connection_state = ConnectionStatus::Disconnected;
                });
                break;
            }

            let _ = cx.try_read_global::<ConnectionState, _>(|state, _cx| {
                if state.active_connection.is_none() {
                    connected = false;
                }
            });

            if !connected {
                break;
            }

            cx.background_executor()
                .timer(Duration::from_millis(1000))
                .await;
        }
    } else {
        let err_msg = match connect_result {
            Err(e) => format!("Connect failed: {}", e),
            Ok(_) => unreachable!(),
        };
        tracing::warn!("{}", err_msg);
        notify_connect_failure(cx, err_msg);
        let _ = cx.update_global::<ConnectionState, _>(|state, _cx| {
            state.active_connection = None;
            state.connection_state = ConnectionStatus::Disconnected;
        });
    }
}

/// Surface a connection failure in the active window so the user
/// doesn't get silently dropped back to the connection list with no
/// indication of what went wrong.
fn notify_connect_failure(cx: &mut AsyncApp, msg: impl Into<String>) {
    let msg: SharedString = msg.into().into();
    let _ = cx.update(|cx| {
        // Push to whichever window is active. If none, log only.
        if let Some(handle) = cx.active_window() {
            let _ = handle.update(cx, |_root, window, cx| {
                window.push_notification((NotificationType::Error, msg.clone()), cx);
            });
        }
    });
}

async fn disconnect_async(db_manager: DatabaseManager, cx: &mut AsyncApp) {
    let _ = cx.update_global::<ConnectionState, _>(|state, _cx| {
        state.active_connection = None;
        state.connection_state = ConnectionStatus::Disconnecting;
    });

    if let Ok(_) = db_manager.disconnect().await {
        let _ = cx.update_global::<ConnectionState, _>(|state, _cx| {
            state.active_connection = None;
            state.connection_state = ConnectionStatus::Disconnected;
        });
    }
}
