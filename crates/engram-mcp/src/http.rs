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

pub struct CallRecord {
    pub name: String,
    pub duration_ms: u64,
    pub ok: bool,
    pub session_id: String,
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
    let response = server.handle_request(&body).await;
    let duration_ms = start.elapsed().as_millis() as u64;

    let tool_name = body["params"]["name"].as_str().unwrap_or("").to_string();
    let ok = response.get("error").is_none();
    if !tool_name.is_empty() {
        (state.on_call)(CallRecord {
            name: tool_name,
            duration_ms,
            ok,
            session_id: session_id.clone(),
        });
    }

    let mut resp_headers = HeaderMap::new();
    if let Ok(v) = HeaderValue::from_str(&session_id) {
        resp_headers.insert("mcp-session-id", v);
    }
    (resp_headers, Json(response))
}
