use axum::{
    extract::{Query, State},
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
    Json, Router,
};
use engram_core::Db;
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt as _;

type SenderMap = Arc<Mutex<HashMap<u64, mpsc::Sender<String>>>>;

#[derive(Clone)]
struct SseState {
    db: Arc<Db>,
    sessions: SenderMap,
    counter: Arc<AtomicU64>,
}

#[derive(Deserialize)]
struct SessionQuery {
    session_id: u64,
}

pub async fn run_sse(db: Arc<Db>, port: u16) -> anyhow::Result<()> {
    let state = SseState {
        db,
        sessions: Arc::new(Mutex::new(HashMap::new())),
        counter: Arc::new(AtomicU64::new(1)),
    };

    let app = Router::new()
        .route("/sse", get(sse_handler))
        .route("/messages", post(messages_handler))
        .with_state(state);

    let addr = format!("127.0.0.1:{port}");
    eprintln!("Engram MCP SSE server listening on http://{addr}");
    eprintln!("  Connect: GET  http://{addr}/sse");
    eprintln!("  Send:    POST http://{addr}/messages?session_id=<id>");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn sse_handler(
    State(state): State<SseState>,
) -> impl axum::response::IntoResponse {
    let session_id = state.counter.fetch_add(1, Ordering::Relaxed);
    let (tx, rx) = mpsc::channel::<String>(32);
    state.sessions.lock().await.insert(session_id, tx);

    let endpoint_data = format!("/messages?session_id={session_id}");
    let sessions_for_cleanup = Arc::clone(&state.sessions);
    let stream = async_stream::stream! {
        yield Ok::<_, std::convert::Infallible>(Event::default().event("endpoint").data(endpoint_data));
        let mut rx_stream = ReceiverStream::new(rx);
        while let Some(msg) = rx_stream.next().await {
            yield Ok(Event::default().event("message").data(msg));
        }
        sessions_for_cleanup.lock().await.remove(&session_id);
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn messages_handler(
    State(state): State<SseState>,
    Query(q): Query<SessionQuery>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let server = crate::server::EngramMcpServer::new(Arc::clone(&state.db));
    let response = server.handle_request(&body).await;

    let sessions = state.sessions.lock().await;
    if let Some(tx) = sessions.get(&q.session_id) {
        let _ = tx.send(serde_json::to_string(&response).unwrap_or_default()).await;
    }

    Json(serde_json::json!({ "ok": true }))
}
