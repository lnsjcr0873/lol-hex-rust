use std::sync::OnceLock;

pub mod config;
pub mod crawler;
pub mod data;
pub mod engine;
pub mod lcu;
pub mod overlay;
pub mod vision;
pub mod web;

pub static GLOBAL_TOKIO_HANDLE: OnceLock<tokio::runtime::Handle> = OnceLock::new();

pub fn spawn_task<F>(future: F)
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    if let Some(handle) = GLOBAL_TOKIO_HANDLE.get() {
        handle.spawn(future);
    } else if let Ok(handle) = tokio::runtime::Handle::try_current() {
        handle.spawn(future);
    } else {
        tracing::error!("Cannot spawn task: Tokio runtime handle unavailable");
    }
}
