# M3 — 임베디드 MCP Supervisor + Manager UI

> **상위 문서**: [overview.md](./overview.md) · **이전**: [m2-dnd-drawer.md](./m2-dnd-drawer.md) · **다음**: [m4-tray-notifications.md](./m4-tray-notifications.md)

**예상 기간**: 1주

## 전제

- [M0](./m0-foundations.md): `engram-mcp` lib + `run_http_with_hook(db, port, on_call, shutdown_rx)` 가 사용 가능
- [M1](./m1-scaffold-board.md), [M2](./m2-dnd-drawer.md): 데스크톱 앱이 정상 작동

## 목표

데스크톱 앱이 켜질 때 임베디드 HTTP MCP 서버가 자동 기동되고, **앱 UI 안에서 시작/정지/재시작/포트 변경/로그·호출 이력을 관리**할 수 있다.

## Scope

### 1. Supervisor 모듈

**`crates/engram-desktop/src/mcp_supervisor.rs`** (신규):

```rust
use engram_core::Db;
use std::{collections::VecDeque, sync::Arc};
use tokio::{sync::{broadcast, oneshot, Mutex}, task::JoinHandle};
use chrono::{DateTime, Utc};

#[derive(Clone, serde::Serialize)]
pub struct SupervisorStatusSnapshot {
    pub running: bool,
    pub port: u16,
    pub started_at: Option<DateTime<Utc>>,
    pub uptime_secs: u64,
    pub call_count: u64,
}

#[derive(Clone, serde::Serialize)]
pub struct CallRecord {
    pub name: String,
    pub args_summary: String,
    pub ok: bool,
    pub duration_ms: u64,
    pub ts: DateTime<Utc>,
    pub session_id: Option<String>,
    pub reason: Option<String>,   // e.g., "timeout"
}

#[derive(Clone, serde::Serialize)]
pub struct LogLine {
    pub level: String,
    pub target: String,
    pub msg: String,
    pub ts: DateTime<Utc>,
}

pub struct McpSupervisor {
    db: Arc<Db>,
    state: Mutex<SupervisorState>,
    log_tx: broadcast::Sender<LogLine>,
    call_log: Mutex<VecDeque<CallRecord>>,  // capacity 200
    call_count: std::sync::atomic::AtomicU64,
}

struct SupervisorState {
    running: bool,
    port: u16,
    started_at: Option<DateTime<Utc>>,
    task: Option<JoinHandle<anyhow::Result<()>>>,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl McpSupervisor {
    pub fn new(db: Arc<Db>) -> Arc<Self> { /* ... */ }

    pub async fn start(self: &Arc<Self>, port: u16) -> anyhow::Result<SupervisorStatusSnapshot> {
        let (tx, rx) = oneshot::channel();
        let hook: engram_mcp::http::CallHook = {
            let me = Arc::clone(self);
            Arc::new(move |rec| me.record_call(rec))
        };
        let db = Arc::clone(&self.db);
        let task = tokio::spawn(async move {
            engram_mcp::http::run_http_with_hook(db, port, hook, rx).await
        });
        let mut s = self.state.lock().await;
        s.running = true; s.port = port;
        s.started_at = Some(Utc::now());
        s.task = Some(task);
        s.shutdown_tx = Some(tx);
        Ok(self.snapshot_locked(&s))
    }

    pub async fn stop(self: &Arc<Self>) -> anyhow::Result<SupervisorStatusSnapshot> {
        let mut s = self.state.lock().await;
        if let Some(tx) = s.shutdown_tx.take() { let _ = tx.send(()); }
        if let Some(handle) = s.task.take() {
            tokio::time::timeout(std::time::Duration::from_secs(3), handle).await.ok();
        }
        s.running = false; s.started_at = None;
        Ok(self.snapshot_locked(&s))
    }

    pub async fn restart(self: &Arc<Self>, port: u16) -> anyhow::Result<SupervisorStatusSnapshot> {
        self.stop().await?;
        self.start(port).await
    }

    pub fn subscribe_logs(&self) -> broadcast::Receiver<LogLine> { self.log_tx.subscribe() }
    pub async fn recent_calls(&self) -> Vec<CallRecord> { self.call_log.lock().await.iter().cloned().collect() }

    fn record_call(self: &Arc<Self>, rec: CallRecord) {
        self.call_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mut log = futures::executor::block_on(self.call_log.lock());
        if log.len() >= 200 { log.pop_front(); }
        log.push_back(rec.clone());
        // Tauri event 발행은 commands 쪽 wrapper 에서 처리
    }
}
```

### 2. 도구 호출 timeout 30초

`engram_mcp::http::run_http_with_hook` 안의 `tools/call` 핸들러에서 dispatch 호출을 `tokio::time::timeout(30s, ...)` 으로 감싼다. 초과 시 `call_log` 에 `ok=false, reason="timeout"` 기록.

### 3. Tauri Commands

**`crates/engram-desktop/src/commands.rs`**:

```rust
#[tauri::command]
pub async fn mcp_status(sup: State<'_, Arc<McpSupervisor>>) -> Result<SupervisorStatusSnapshot, String>;

#[tauri::command]
pub async fn mcp_start(sup: State<'_, Arc<McpSupervisor>>, port: u16) -> Result<SupervisorStatusSnapshot, String>;

#[tauri::command]
pub async fn mcp_stop(sup: State<'_, Arc<McpSupervisor>>) -> Result<SupervisorStatusSnapshot, String>;

#[tauri::command]
pub async fn mcp_restart(sup: State<'_, Arc<McpSupervisor>>, port: u16) -> Result<SupervisorStatusSnapshot, String>;

#[tauri::command]
pub async fn mcp_recent_calls(sup: State<'_, Arc<McpSupervisor>>) -> Result<Vec<CallRecord>, String>;

#[tauri::command]
pub async fn mcp_set_autostart(app: AppHandle, on: bool) -> Result<(), String>;
```

### 4. Tauri 이벤트

| 이벤트 이름 | trigger | payload |
|---|---|---|
| `mcp://status` | start/stop/restart 직후 + heartbeat (10s) | `SupervisorStatusSnapshot` |
| `mcp://log`    | tracing broadcast Layer 가 로그 발행할 때 | `LogLine` |
| `mcp://call`   | hook 이 호출 종료 시 | `CallRecord` |

**브로드캐스트 → Tauri event 펌프** (`main.rs`):
```rust
let app_handle = app.handle().clone();
let mut log_rx = supervisor.subscribe_logs();
tauri::async_runtime::spawn(async move {
    while let Ok(line) = log_rx.recv().await {
        let _ = app_handle.emit("mcp://log", &line);
    }
});
```

호출 hook 안에서도 동일하게 `emit("mcp://call", ...)`.

### 5. tracing broadcast Layer

데스크톱 main 에서 layered subscriber:

```rust
let log_tx = supervisor.log_sender();  // broadcast::Sender<LogLine>

tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer())
    .with(tracing_subscriber::EnvFilter::from_default_env())
    .with(BroadcastLayer { tx: log_tx })   // 신규 Layer
    .init();
```

`BroadcastLayer` 는 모든 `Event` 를 `LogLine` 으로 변환해 송신.

### 6. 자동 기동 + 설정 영속화

**`crates/engram-desktop/src/settings.rs`**:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DesktopSettings {
    pub mcp: McpSettings,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct McpSettings {
    pub autostart: bool,
    pub host: String,        // "127.0.0.1"
    pub port: u16,           // 3456
    pub transport: String,   // "http"
}

pub fn load() -> anyhow::Result<DesktopSettings> { /* ~/.engram/desktop.toml */ }
pub fn save(s: &DesktopSettings) -> anyhow::Result<()> { /* atomic write */ }
pub fn set_autostart(on: bool) -> anyhow::Result<()> { /* ... */ }
```

`main.rs` setup 에서 `settings::load()` 후 `if autostart { supervisor.start(port).await }`.

### 7. Graceful shutdown

```rust
.on_window_event(|window, event| {
    if let tauri::WindowEvent::CloseRequested { .. } = event {
        let sup = window.app_handle().state::<Arc<McpSupervisor>>().inner().clone();
        tauri::async_runtime::block_on(async move { let _ = sup.stop().await; });
    }
})
```

### 8. McpManager.tsx UI

라우트 진입 시:
- `mcp_status` 1회 fetch + `mcp://status` 이벤트 구독
- `mcp_recent_calls` 1회 fetch + `mcp://call` 이벤트로 prepend
- `mcp://log` 구독 → tail 100줄 유지

컴포넌트 구조 (`ui/src/routes/McpManager.tsx`):
- 상태 패널 (●/○ + uptime + endpoint + 복사 버튼)
- 자동 기동 토글 (`mcp_set_autostart`)
- 시작/정지/재시작 버튼
- "Claude Code 설정 보기" modal (jsonc 스니펫)
- 최근 호출 테이블 (ts/name/ok/duration_ms)
- 로그 tail (level color 적용)

## 변경 파일 목록

```
crates/engram-mcp/src/http.rs                         (M)  on_call hook 호출 + 30s timeout
crates/engram-desktop/src/main.rs                     (M)  supervisor manage, autostart, event pump
crates/engram-desktop/src/commands.rs                 (M)  mcp_* 명령
crates/engram-desktop/src/mcp_supervisor.rs           (+)  신규
crates/engram-desktop/src/settings.rs                 (M)  실제 구현 (M1 stub 교체)
crates/engram-desktop/src/tracing_layer.rs            (+)  BroadcastLayer
crates/engram-desktop/Cargo.toml                      (M)  socket2, futures, etc.
crates/engram-desktop/ui/src/                         (+/M)
  routes/McpManager.tsx                               (+)
  hooks/useMcpStatus.ts                               (+)
  hooks/useMcpLogs.ts                                 (+)
  hooks/useMcpCalls.ts                                (+)
  ipc/invoke.ts                                       (M)  mcp_* 래퍼
  store/ui.ts                                         (M)  view='mcp'
```

## Verification

1. **빌드**
   ```bash
   cargo build -p engram-desktop
   pnpm --filter engram-desktop-ui build
   ```
2. **시나리오**
   - 앱 기동 → 자동 기동 토글이 켜져 있으면 즉시 `MCP ●` 표시
   - `curl -X POST http://127.0.0.1:3456/mcp -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'` → 200
   - McpManager 의 "최근 호출" 에 `initialize` 1행 추가, ts/duration_ms 정상
   - `[정지]` 클릭 → ○ 변경 + curl 실패
   - 포트 3457 로 변경 + `[재시작]` → 즉시 재바인딩 (SO_REUSEADDR 효과)
   - 로그 tail 에 `client connected`, `dispatch ok` 등이 실시간 표시
   - 잘못된 도구 이름 호출 시 `call_log` 에 `ok=false` 기록
   - 30초 이상 걸리는 더미 도구 (테스트용) → timeout reason 기록
3. **종료 시**
   - 메인 윈도우 닫기 → `stop()` 호출 → 3초 내 listener 해제 (`lsof -i :3456` 비어 있음)
4. **WAL 동시성**
   - 데스크톱에서 카드 이동 (쓰기) + curl 로 `task_next` (읽기) 동시 → 양쪽 성공, `busy_timeout` 도달 X

## Out of Scope

- 트레이 UI / 알림 (→ M4)
- SSE GET 채널의 server→client notification (Phase 2 이후)
- 인증 (옵션 — Opus 권고)

## 완료 기준

- [x] 자동 기동/수동 기동 둘 다 작동
- [x] 시작/정지/재시작이 UI 에서 즉시 반영
- [x] 호출 이력 200개 ring buffer 유지
- [x] 로그 tail 실시간 갱신
- [x] 30초 timeout 동작
- [x] 윈도우 종료 시 graceful shutdown
- [x] `~/.engram/desktop.toml` 영속화
