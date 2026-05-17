use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode},
    routing::post,
    Json, Router,
};
use engram_core::Db;
use serde_json::Value;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::{oneshot, Mutex};
use uuid::Uuid;

#[derive(Clone, serde::Serialize)]
pub struct CallRecord {
    pub name: String,
    pub args_summary: String,
    pub ok: bool,
    pub duration_ms: u64,
    pub ts: chrono::DateTime<chrono::Utc>,
    pub session_id: Option<String>,
    pub reason: Option<String>,
}

pub type CallHook = Arc<dyn Fn(CallRecord) + Send + Sync>;

#[derive(Clone)]
struct HttpState {
    db: Arc<Db>,
    sessions: Arc<Mutex<HashMap<String, ()>>>,
    on_call: CallHook,
}

pub async fn run_http_with_hook(
    db: Arc<Db>,
    port: u16,
    on_call: CallHook,
    shutdown: oneshot::Receiver<()>,
) -> anyhow::Result<()> {
    let state = HttpState {
        db,
        sessions: Arc::new(Mutex::new(HashMap::new())),
        on_call,
    };

    let app = Router::new()
        .route("/mcp", post(post_handler).get(get_not_allowed))
        .with_state(state);

    // SO_REUSEADDR enables fast restart on the same port
    use socket2::{Domain, Socket, Type};
    let socket = Socket::new(Domain::IPV4, Type::STREAM, None)?;
    socket.set_reuse_address(true)?;
    socket.bind(&SocketAddr::from(([127, 0, 0, 1], port)).into())?;
    socket.listen(128)?;
    socket.set_nonblocking(true)?;
    let listener = tokio::net::TcpListener::from_std(socket.into())?;

    tracing::info!("Engram MCP HTTP server listening on http://127.0.0.1:{port}/mcp");
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = shutdown.await;
        })
        .await?;
    Ok(())
}

pub async fn run_http(db: Arc<Db>, port: u16) -> anyhow::Result<()> {
    let (_tx, rx) = oneshot::channel::<()>();
    run_http_with_hook(db, port, Arc::new(|_| {}), rx).await
}

async fn get_not_allowed() -> StatusCode {
    StatusCode::METHOD_NOT_ALLOWED
}

async fn post_handler(
    State(state): State<HttpState>,
    req_headers: HeaderMap,
    Json(body): Json<Value>,
) -> (HeaderMap, Json<Value>) {
    let session_id = req_headers
        .get("mcp-session-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    state.sessions.lock().await.entry(session_id.clone()).or_insert(());

    let start = std::time::Instant::now();
    let server = crate::server::EngramMcpServer::new(Arc::clone(&state.db));
    let timeout_result = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        server.handle_request(&body),
    )
    .await;
    let duration_ms = start.elapsed().as_millis() as u64;

    let (response, ok, reason) = match timeout_result {
        Ok(resp) => {
            let ok = resp.get("error").is_none();
            (resp, ok, None)
        }
        Err(_) => {
            let resp = serde_json::json!({
                "jsonrpc": "2.0",
                "id": body["id"],
                "error": { "code": -32000, "message": "Tool call timed out (30s)" }
            });
            (resp, false, Some("timeout".to_string()))
        }
    };

    let tool_name = body["params"]["name"].as_str().unwrap_or("").to_string();
    let args_summary = body["params"]["arguments"]
        .to_string()
        .chars()
        .take(100)
        .collect::<String>();
    if !tool_name.is_empty() {
        (state.on_call)(CallRecord {
            name: tool_name,
            args_summary,
            ok,
            duration_ms,
            ts: chrono::Utc::now(),
            session_id: Some(session_id.clone()),
            reason,
        });
    }

    let mut resp_headers = HeaderMap::new();
    if let Ok(v) = HeaderValue::from_str(&session_id) {
        resp_headers.insert("mcp-session-id", v);
    }
    (resp_headers, Json(response))
}
