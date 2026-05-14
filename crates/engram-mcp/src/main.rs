mod server;
mod tools;

use engram_core::Db;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr) // MCP는 stdout을 JSON-RPC에 사용하므로 stderr로 로깅
        .init();

    let db = Arc::new(Db::open_default().await?);
    tracing::info!("Engram MCP Server starting...");

    server::EngramMcpServer::new(db)
        .run_stdio()
        .await
}
