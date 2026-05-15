# M0 — 선행 정비 (Foundations)

> **상위 문서**: [overview.md](./overview.md) · **다음**: [m1-scaffold-board.md](./m1-scaffold-board.md)
>
> Opus 아키텍트 리뷰의 Critical 3건 + lib 분리 권고를 모두 M0 로 끌어와 별도 PR 로 처리. 다른 모든 마일스톤이 이 변경을 가정한다.

**예상 기간**: 2~3일

## 목표

데스크톱 앱이 안전하게 임베디드 MCP 서버를 호스팅하기 위한 코어/MCP 레이어 정비.

## Scope

### 1. `engram-mcp` lib + bin 듀얼 패키지 전환

**Why**: 데스크톱 앱이 `engram_mcp::http::run_http_with_hook` 등을 라이브러리로 사용해야 함. 별도 프로세스 호출 X.

**변경**:
- `crates/engram-mcp/Cargo.toml`:
  ```toml
  [lib]
  name = "engram_mcp"
  path = "src/lib.rs"

  [[bin]]
  name = "engram-mcp"
  path = "src/main.rs"
  ```
- `crates/engram-mcp/src/lib.rs` (신규):
  ```rust
  pub mod tools;
  pub mod server;
  pub mod http;
  // 주의: tracing_subscriber 초기화 금지 — entry-point 에서만
  ```
- `crates/engram-mcp/src/main.rs`: 인자 파싱 + `engram_mcp::*` 호출로 슬림화. `tracing_subscriber::fmt().init()` 는 main.rs 에 유지.

### 2. SSE 폐기 → Streamable HTTP

**Why**: MCP 사양(2025-03-26+) 에서 SSE deprecated, Streamable HTTP 가 표준. axum 기반은 그대로 활용.

**변경**:
- `crates/engram-mcp/src/sse.rs` **삭제**
- `crates/engram-mcp/src/http.rs` (신규):
  ```rust
  use axum::{extract::State, routing::{post, get}, Json, Router};
  use std::{net::SocketAddr, sync::Arc};
  use tokio::sync::oneshot;

  pub type CallHook = Arc<dyn Fn(CallRecord) + Send + Sync>;

  pub async fn run_http_with_hook(
      db: Arc<engram_core::Db>,
      port: u16,
      on_call: CallHook,
      shutdown: oneshot::Receiver<()>,
  ) -> anyhow::Result<()> {
      let state = HttpState { db, sessions: Default::default(), on_call };
      let app = Router::new()
          .route("/mcp", post(post_handler).get(get_handler))
          .with_state(state);

      // SO_REUSEADDR for fast restart on same port
      let socket = socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::STREAM, None)?;
      socket.set_reuse_address(true)?;
      socket.bind(&SocketAddr::from(([127, 0, 0, 1], port)).into())?;
      socket.listen(128)?;
      let listener = tokio::net::TcpListener::from_std(socket.into())?;

      axum::serve(listener, app)
          .with_graceful_shutdown(async move { let _ = shutdown.await; })
          .await?;
      Ok(())
  }
  ```
- 핸들러:
  - `POST /mcp`: `Json<JsonRpcRequest>` → `server::EngramMcpServer::handle_request` 호출
  - `GET /mcp`: 서버→클라이언트 알림용 SSE. M0 에서는 `405 Method Not Allowed` 반환 OK
  - `Mcp-Session-Id` 헤더로 세션 추적. 첫 요청에 UUID 발급
- `run_http_with_hook` 에서 모든 `tools/call` 응답 전후로 `on_call({ name, ts, duration_ms, ok, session_id })` 호출

### 3. `Db::open` 풀 사이즈 명시 (Critical C2)

**Why**: sqlx 0.7 기본 10 connection. WAL + busy_timeout 5초 환경에서 데스크톱+HTTP+watcher 동시 접근 시 stall 위험. SQLite 는 단일 writer 라 5 connection 으로 충분.

**변경** (`crates/engram-core/src/repository/mod.rs`):
```rust
let pool = SqlitePoolOptions::new()
    .max_connections(5)
    .connect_with(options)
    .await?;
```

`open_in_memory` 도 동일하게 적용.

### 4. `changed_by` 파라미터 강제 (Critical C1)

**Why**: Demo Gate 정책의 핵심 fallback 이 `history.changed_by` 감사인데, 현재 `"agent"` 하드코딩. 데스크톱 호출도 `"agent"` 로 기록되어 audit trail 무의미.

**대상 메서드** — 모두 `changed_by: &str` 파라미터 추가:
- `crates/engram-core/src/repository/issue.rs::issue_update`
- `crates/engram-core/src/repository/task.rs::task_update`
- `crates/engram-core/src/repository/epic.rs::epic_update`
- `crates/engram-core/src/repository/sprint.rs::sprint_update`
- `crates/engram-core/src/repository/note.rs::note_resolve`

**호출처 갱신**:
- `engram-mcp/src/tools/*` → `"agent"` 전달
- `engram-cli/src/commands/*` → `"user"` 전달 (사람이 실행하므로)
- `engram-desktop/src/commands.rs` → `"user"` 전달 (M1 부터)
- 모든 기존 테스트 (`workflow_test.rs`, `dispatch_test.rs`, repository inline tests) → 호출처 일괄 갱신

### 5. tracing 위생

- `engram-mcp` lib 코드 안에서 `tracing_subscriber::init()` 호출 금지 (lib 는 facade 사용만)
- `engram-mcp/src/main.rs` 와 (M1 의) `engram-desktop/src/main.rs` 두 entry-point 에서만 init
- 데스크톱은 fmt Layer + broadcast Layer 합성 (M3 에서 broadcast 채널 추가)

### 6. ADR 작성

- `docs/adr/0008-embedded-mcp-supervisor.md`:
  - Status: Accepted
  - Decision: 데스크톱 앱이 HTTP MCP 서버를 같은 프로세스 내 tokio task 로 호스팅. 별도 프로세스 사용 안 함
  - Consequences: 단일 Db pool 공유 (WAL + max_connections=5 로 안전), graceful shutdown 가능, 같은 runtime 사용
  - Trade-offs: 도구 호출이 starve 되지 않도록 30초 timeout 필요 (M3 에서 구현)

## 변경 파일 목록

```
crates/engram-mcp/Cargo.toml                          (M)  [lib] 섹션 추가
crates/engram-mcp/src/lib.rs                          (+)  신규
crates/engram-mcp/src/main.rs                         (M)  shim 화
crates/engram-mcp/src/sse.rs                          (-)  삭제
crates/engram-mcp/src/http.rs                         (+)  신규
crates/engram-mcp/src/tools/*                         (M)  changed_by="agent" 전달
crates/engram-core/src/repository/mod.rs              (M)  max_connections(5)
crates/engram-core/src/repository/{issue,task,epic,sprint,note}.rs  (M)  changed_by 파라미터
crates/engram-core/src/repository/history.rs          (M)  필요 시 helper 정리
crates/engram-core/tests/workflow_test.rs             (M)  호출처 갱신
crates/engram-mcp/src/tools/dispatch_test.rs          (M)  호출처 갱신
crates/engram-cli/src/commands/*                      (M)  changed_by="user" 전달
docs/adr/0008-embedded-mcp-supervisor.md              (+)  신규
Cargo.toml (workspace)                                (M)  필요 시 socket2 의존성 등록
```

## Verification

### 자동 테스트
```bash
cargo build --workspace
cargo test --workspace            # 기존 36건 + 신규 history actor 테스트 1건 = 37+ green
```

### 신규 테스트 (history actor 검증)
`crates/engram-core/tests/workflow_test.rs` 에 1건 추가:

```rust
#[tokio::test]
async fn test_history_records_changed_by_actor() {
    let db = Db::open_in_memory().await.unwrap();
    let sprint = db.sprint_create(/* ... */).await.unwrap();
    db.sprint_update(sprint.id, /* status=Active */, "user").await.unwrap();

    let epic = db.epic_create(/* ... */).await.unwrap();
    let issue = db.issue_create(/* epic_id=epic.id */).await.unwrap();

    db.issue_update(issue.id, /* status=Ready */, "agent").await.unwrap();
    db.issue_update(issue.id, /* status=Working */, "agent").await.unwrap();
    db.issue_update(issue.id, /* status=Demo */, "agent").await.unwrap();
    db.issue_update(issue.id, /* status=Finished */, "user").await.unwrap();

    let history = db.history_list(EntityType::Issue, issue.id).await.unwrap();
    let last = history.iter().rfind(|h| h.field == "status").unwrap();
    assert_eq!(last.changed_by, "user", "finished 전이는 사용자가 한 것으로 기록되어야 함");
}
```

### MCP HTTP 수동 검증
```bash
cargo run -p engram-mcp -- --transport=http --port=3456 &
curl -X POST http://127.0.0.1:3456/mcp \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'
# → 200 + Mcp-Session-Id 응답 헤더
```

## Out of Scope

- `engram-desktop` 크레이트 생성 (→ M1)
- Tauri 관련 일체 (→ M1+)
- Supervisor 모듈 (→ M3)
- McpManager UI (→ M3)

## 완료 기준

- [x] `cargo build --workspace` clean
- [x] `cargo test --workspace` 기존 36건 + 신규 1건 = 37+ green
- [x] `engram-mcp --transport=http` 가 SSE 헤더 없이 JSON-RPC 응답 (http.rs 구현)
- [x] HTTP 서버에 SIGTERM 보내면 graceful shutdown (oneshot 채널)
- [x] 같은 포트 즉시 재시작 가능 (SO_REUSEADDR — http.rs 적용)
- [x] `history.changed_by` 가 호출처별로 다르게 기록 (test_history_records_changed_by_actor 통과)
- [x] ADR-0008 머지 (ADR-0006, 0007, 0008 작성 완료)
