use engram_core::Db;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let args: Vec<String> = std::env::args().collect();
    let transport = args.iter()
        .find(|a| a.starts_with("--transport="))
        .map(|a| a.trim_start_matches("--transport=").to_string())
        .unwrap_or_else(|| "stdio".to_string());
    let port: u16 = args.iter()
        .find(|a| a.starts_with("--port="))
        .and_then(|a| a.trim_start_matches("--port=").parse().ok())
        .unwrap_or(3456);

    let db = Arc::new(Db::open_default().await?);

    match transport.as_str() {
        "http" => {
            tracing::info!("Starting Engram MCP HTTP server on port {port}");
            engram_mcp::http::run_http(db, port).await
        }
        _ => {
            tracing::info!("Starting Engram MCP stdio server");
            engram_mcp::server::EngramMcpServer::new(db).run_stdio().await
        }
    }
}
