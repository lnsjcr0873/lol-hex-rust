#![recursion_limit = "512"]

use std::sync::OnceLock;

pub mod config;
pub mod crawler;
pub mod data;
pub mod engine;
pub mod lcu;
pub mod overlay;
pub mod sgp;
pub mod vision;
pub mod web;

pub static GLOBAL_TOKIO_HANDLE: OnceLock<tokio::runtime::Handle> = OnceLock::new();

/// Sets the global Tokio runtime handle used across threads (e.g. Win32 Hooks, GDI rendering).
pub fn init_tokio_handle(handle: tokio::runtime::Handle) {
    let _ = GLOBAL_TOKIO_HANDLE.set(handle);
}

/// Retrieves the global Tokio runtime handle or falls back to current thread handle.
pub fn get_tokio_handle() -> Option<tokio::runtime::Handle> {
    GLOBAL_TOKIO_HANDLE.get().cloned().or_else(|| tokio::runtime::Handle::try_current().ok())
}

pub fn spawn_task<F>(future: F)
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    if let Some(handle) = get_tokio_handle() {
        handle.spawn(future);
    } else {
        tracing::error!("Cannot spawn task: Tokio runtime handle unavailable");
    }
}
