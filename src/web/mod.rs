pub mod assets;
pub mod routes;
pub mod ws;

pub use routes::{create_router, AppState};
pub use ws::WsBroadcaster;

use std::net::SocketAddr;
use tracing::info;

pub async fn start_web_server(state: AppState, port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = create_router(state);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("🌐 HexBrain Web UI Server listening on http://{}", addr);
    println!("🌐 HexBrain Web 控制中心已启动: http://127.0.0.1:{}", port);
    axum::serve(listener, app).await?;
    Ok(())
}
