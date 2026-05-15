use chrono::{DateTime, Utc};
use engram_core::Db;
use engram_mcp::http::{CallHook, CallRecord};
use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
};
use tokio::{
    sync::{broadcast, oneshot, Mutex},
    task::JoinHandle,
};

#[derive(Clone, serde::Serialize)]
pub struct SupervisorStatusSnapshot {
    pub running: bool,
    pub port: u16,
    pub started_at: Option<DateTime<Utc>>,
    pub uptime_secs: u64,
    pub call_count: u64,
    pub autostart: bool,
}

#[derive(Clone, serde::Serialize)]
pub struct LogLine {
    pub level: String,
    pub target: String,
    pub msg: String,
    pub ts: DateTime<Utc>,
}

struct SupervisorState {
    running: bool,
    port: u16,
    started_at: Option<DateTime<Utc>>,
    task: Option<JoinHandle<anyhow::Result<()>>>,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

pub struct McpSupervisor {
    db: Arc<Db>,
    state: Mutex<SupervisorState>,
    pub log_tx: broadcast::Sender<LogLine>,
    call_broadcast_tx: broadcast::Sender<CallRecord>,
    call_log: Mutex<VecDeque<CallRecord>>,
    call_count: AtomicU64,
    autostart: AtomicBool,
}

impl McpSupervisor {
    pub fn new(db: Arc<Db>, autostart: bool) -> Arc<Self> {
        let (log_tx, _) = broadcast::channel(256);
        let (call_broadcast_tx, _) = broadcast::channel(256);
        Arc::new(Self {
            db,
            state: Mutex::new(SupervisorState {
                running: false,
                port: 3456,
                started_at: None,
                task: None,
                shutdown_tx: None,
            }),
            log_tx,
            call_broadcast_tx,
            call_log: Mutex::new(VecDeque::with_capacity(200)),
            call_count: AtomicU64::new(0),
            autostart: AtomicBool::new(autostart),
        })
    }

    pub async fn start(self: &Arc<Self>, port: u16) -> anyhow::Result<SupervisorStatusSnapshot> {
        let mut s = self.state.lock().await;
        if s.running {
            anyhow::bail!("MCP server already running on port {}", s.port);
        }

        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let hook: CallHook = {
            let me = Arc::clone(self);
            Arc::new(move |rec: CallRecord| {
                me.record_call(rec);
            })
        };
        let db = Arc::clone(&self.db);
        let me_monitor = Arc::clone(self);
        let task = tokio::spawn(async move {
            let result = engram_mcp::http::run_http_with_hook(db, port, hook, shutdown_rx).await;
            // If task exits unexpectedly (before stop() was called), flip state
            let mut s = me_monitor.state.lock().await;
            if s.running {
                s.running = false;
                s.started_at = None;
                tracing::warn!("MCP HTTP server exited unexpectedly");
            }
            result
        });

        s.running = true;
        s.port = port;
        s.started_at = Some(Utc::now());
        s.task = Some(task);
        s.shutdown_tx = Some(shutdown_tx);
        Ok(self.snapshot_locked(&s))
    }

    pub async fn stop(self: &Arc<Self>) -> anyhow::Result<SupervisorStatusSnapshot> {
        let mut s = self.state.lock().await;
        if let Some(tx) = s.shutdown_tx.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = s.task.take() {
            tokio::time::timeout(std::time::Duration::from_secs(3), handle)
                .await
                .ok();
        }
        s.running = false;
        s.started_at = None;
        Ok(self.snapshot_locked(&s))
    }

    pub async fn restart(self: &Arc<Self>, port: u16) -> anyhow::Result<SupervisorStatusSnapshot> {
        self.stop().await?;
        self.start(port).await
    }

    pub async fn status(self: &Arc<Self>) -> SupervisorStatusSnapshot {
        let s = self.state.lock().await;
        self.snapshot_locked(&s)
    }

    pub fn subscribe_logs(&self) -> broadcast::Receiver<LogLine> {
        self.log_tx.subscribe()
    }

    pub async fn recent_calls(&self) -> Vec<CallRecord> {
        self.call_log.lock().await.iter().cloned().collect()
    }

    pub fn log_sender(&self) -> broadcast::Sender<LogLine> {
        self.log_tx.clone()
    }

    pub fn record_call(self: &Arc<Self>, rec: CallRecord) {
        self.call_count.fetch_add(1, Ordering::Relaxed);
        let _ = self.call_broadcast_tx.send(rec.clone()); // ignore if no receivers
        // Use try_lock to avoid blocking; if busy, skip recording this call
        if let Ok(mut log) = self.call_log.try_lock() {
            if log.len() >= 200 {
                log.pop_front();
            }
            log.push_back(rec);
        }
    }

    pub fn call_broadcast_sender(&self) -> broadcast::Sender<CallRecord> {
        self.call_broadcast_tx.clone()
    }

    pub fn set_autostart(&self, on: bool) {
        self.autostart.store(on, Ordering::Relaxed);
    }

    fn snapshot_locked(&self, s: &SupervisorState) -> SupervisorStatusSnapshot {
        let uptime_secs = s
            .started_at
            .map(|t| Utc::now().signed_duration_since(t).num_seconds().max(0) as u64)
            .unwrap_or(0);
        SupervisorStatusSnapshot {
            running: s.running,
            port: s.port,
            started_at: s.started_at,
            uptime_secs,
            call_count: self.call_count.load(Ordering::Relaxed),
            autostart: self.autostart.load(Ordering::Relaxed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engram_core::Db;

    async fn make_supervisor() -> Arc<McpSupervisor> {
        let db = Arc::new(Db::open_in_memory().await.unwrap());
        McpSupervisor::new(db, false)
    }

    #[tokio::test]
    async fn test_supervisor_initial_state_is_stopped() {
        let sup = make_supervisor().await;
        let snap = sup.status().await;
        assert!(!snap.running);
        assert_eq!(snap.uptime_secs, 0);
        assert_eq!(snap.call_count, 0);
    }

    #[tokio::test]
    async fn test_supervisor_start_sets_running_true() {
        let sup = make_supervisor().await;
        let snap = sup.start(13456).await.unwrap();
        assert!(snap.running);
        assert_eq!(snap.port, 13456);
        // Cleanup
        sup.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_supervisor_stop_sets_running_false() {
        let sup = make_supervisor().await;
        sup.start(13457).await.unwrap();
        let snap = sup.stop().await.unwrap();
        assert!(!snap.running);
    }

    #[tokio::test]
    async fn test_supervisor_restart_changes_port() {
        let sup = make_supervisor().await;
        sup.start(13458).await.unwrap();
        let snap = sup.restart(13459).await.unwrap();
        assert!(snap.running);
        assert_eq!(snap.port, 13459);
        sup.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_call_log_ring_buffer_capped_at_200() {
        let sup = make_supervisor().await;
        for i in 0..210u64 {
            sup.record_call(CallRecord {
                name: format!("tool_{i}"),
                args_summary: "{}".to_string(),
                ok: true,
                duration_ms: 1,
                ts: Utc::now(),
                session_id: None,
                reason: None,
            });
        }
        let calls = sup.recent_calls().await;
        assert_eq!(calls.len(), 200);
        // Should have the most recent 200
        assert_eq!(calls[0].name, "tool_10");
    }

    #[tokio::test]
    async fn test_stop_when_not_running_is_safe() {
        let sup = make_supervisor().await;
        // Should not panic or error
        let snap = sup.stop().await.unwrap();
        assert!(!snap.running);
    }
}
