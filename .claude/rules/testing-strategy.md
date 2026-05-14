# Rule: 테스트 전략

## 원칙

- **모든 테스트는 in-memory SQLite (`:memory:`)** — 디스크 / 네트워크 / 외부 프로세스 의존 금지.
- 한 테스트 = 새 풀. 테스트 간 상태 공유 없음.
- `#[tokio::test]` + `async fn`. `tokio` 는 workspace 의존성으로 이미 포함.
- 이름: `test_<feature>_<expected_behavior>` (snake_case).
  - 예: `test_session_restore_filters_by_project`, `test_task_next_skips_blocked`.

## 위치

| 종류 | 경로 |
|------|------|
| Repository 단위 테스트 | `crates/engram-core/src/repository/<entity>.rs` 하단 `#[cfg(test)] mod tests {}` |
| 워크플로 통합 테스트 | `crates/engram-core/tests/workflow_test.rs` (없으면 신규) |
| MCP 디스패치 / 직렬화 테스트 | `crates/engram-mcp/src/tools/<area>.rs` 하단 `#[cfg(test)] mod tests {}` |
| CLI 파싱 테스트 | `crates/engram-cli/src/commands/<area>.rs` 하단 |

## 표준 셋업 헬퍼

각 테스트 모듈 상단에 둔다:

```rust
use engram_core::Db;

async fn setup_db() -> Db {
    Db::open(":memory:").await.expect("open in-memory db")
}
```

`Db::open` 이 `sqlx::migrate!("./migrations").run(&pool)` 를 호출하므로 별도 schema setup 불필요.

## 워크플로 테스트 패턴

`tests/workflow_test.rs` 는 **실제 사용 시나리오** 단위로 작성한다:

```rust
#[tokio::test]
async fn test_full_sprint_workflow() {
    let db = setup_db().await;

    // 1. sprint → epic → issue(draft) → approve → tasks → note → session_restore
    let sprint = db.sprint_create(...).await.unwrap();
    let epic   = db.epic_create(...).await.unwrap();
    // ...
    let snapshot = db.session_restore(Some("xpert-da-web")).await.unwrap();

    assert_eq!(snapshot.active_epics.len(), 1);
    assert!(snapshot.next_action.is_some());
}
```

- 한 테스트가 5~20 스텝의 시나리오를 검증. mock 사용 금지 (실제 DB).
- 시나리오마다 별도 함수로 분리: 풀 워크플로 / 크로스 프로젝트 블로킹 / 스코프 팽창 감지 등.

## MCP 도구 테스트

JSON-RPC round-trip:

```rust
#[tokio::test]
async fn test_session_restore_via_dispatch() {
    let db = Arc::new(setup_db().await);
    let args = json!({ "project_key": "test-proj" });
    let result = tools::dispatch(db, "session_restore", &args).await.unwrap();
    assert!(result["sprint_id"].is_i64());
}
```

도구 정의 자체의 직렬화는 `all_tool_definitions()` 결과를 `serde_json::to_string` 으로 검증.

## 어서션 메시지

`assert_eq!(actual, expected)` 만으로는 디버깅이 어렵다 — 의미를 한 줄 추가:

```rust
assert_eq!(snapshot.pending_drafts.len(), 1, "draft 이슈 한 건이 pending 으로 잡혀야 함");
```

## 금지 사항

- `unwrap()` 남발 OK (테스트 한정), 단 의미 있는 위치만.
- `println!` / `dbg!` 는 머지 전 제거.
- `sleep` / `tokio::time::sleep` 금지 — 실제 시간 의존 테스트는 작성하지 않는다.
- 실제 `~/.engram/engram.db` 를 건드리는 테스트 금지.
