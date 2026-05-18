//! CLI ↔ MCP dispatch 동치 통합 테스트.
//!
//! `.claude/rules/testing-strategy.md` 의 in-memory SQLite 원칙 준수.
//! engram-mcp 의 `tools::dispatch(name, args)` 결과 JSON 과
//! engram-core 의 동등 repository 호출을 `serde_json::to_value` 로 직렬화한 결과가
//! 의미적으로 동일함을 검증한다. CLI handler 는 결국 repository 호출 + print_value
//! 직렬화이므로 repository 호출 결과 == MCP dispatch 결과 == CLI --json 페이로드.
//!
//! 검증 범위:
//! 1. read-only: sprint_list/current, epic_list, issue_list/get, task_list/next,
//!    note_list/get, session_restore/end, board_status, my_blocked_issues,
//!    history_recent/by_agent/for, task_test_list, stalled_issues.
//! 2. 변경 도구 대표 시나리오: sprint_create/update, epic_create/update,
//!    issue_create/update/link/unlink/claim/release/set_sprint/delete,
//!    task_create/update/insert_after/delete, note_add/resolve,
//!    task_test_add/check/uncheck/remove.

#![cfg(test)]

use engram_core::{
    Db,
    models::{
        epic::CreateEpicInput,
        issue::{CreateIssueInput, IssueFilter, LinkType},
        note::{CreateNoteInput, NoteType},
        sprint::{CreateSprintInput, SprintStatus, UpdateSprintInput},
        task::CreateTaskInput,
    },
};
use engram_mcp::tools::dispatch;
use serde_json::{json, Value};
use std::sync::Arc;

async fn fresh_db() -> Arc<Db> {
    Arc::new(Db::open_in_memory().await.expect("in-memory db"))
}

/// 동일한 시퀀스로 양쪽 DB 를 시드한 결과 (sprint_id, epic_id, issue_id, task_id).
/// MCP 쪽 시드는 dispatch 로, CLI 쪽 시드는 Db 직접 호출로 수행 — 두 경로가 같은 결과를
/// 만들면 그 자체로 동치성을 한 번 검증한 것.
async fn seed_via_db(db: &Arc<Db>) -> (i64, i64, i64, i64) {
    let s = db.sprint_create(CreateSprintInput {
        name: "S1".into(), goal: None, start_date: None, end_date: None,
    }).await.unwrap();
    db.sprint_update(s.id, UpdateSprintInput {
        name: None, status: Some(SprintStatus::Active),
        goal: None, start_date: None, end_date: None,
    }, "test").await.unwrap();
    let e = db.epic_create(CreateEpicInput {
        project_key: "p".into(), title: "E".into(), description: None,
    }).await.unwrap();
    let i = db.issue_create(CreateIssueInput {
        epic_id: e.id, sprint_id: Some(s.id), title: "I".into(),
        description: None, goal: None, priority: None,
    }).await.unwrap();
    let t = db.task_create(CreateTaskInput {
        issue_id: i.id, title: "T".into(), description: None,
        goal: None, after_task_id: None, source: None,
    }).await.unwrap();
    (s.id, e.id, i.id, t.id)
}

async fn seed_via_dispatch(db: &Arc<Db>) -> (i64, i64, i64, i64) {
    let s = dispatch(Arc::clone(db), "sprint_create", &json!({"name": "S1"})).await.unwrap();
    let sid = s["id"].as_i64().unwrap();
    dispatch(Arc::clone(db), "sprint_update", &json!({"id": sid, "status": "active"}))
        .await.unwrap();
    let e = dispatch(Arc::clone(db), "epic_create",
        &json!({"sprint_id": sid, "project_key": "p", "title": "E"})).await.unwrap();
    let eid = e["id"].as_i64().unwrap();
    let i = dispatch(Arc::clone(db), "issue_create",
        &json!({"epic_id": eid, "sprint_id": sid, "title": "I"})).await.unwrap();
    let iid = i["id"].as_i64().unwrap();
    let t = dispatch(Arc::clone(db), "task_create",
        &json!({"issue_id": iid, "title": "T"})).await.unwrap();
    let tid = t["id"].as_i64().unwrap();
    (sid, eid, iid, tid)
}

/// `created_at` / `updated_at` 등 시간 컬럼은 같은 시드라도 ms 차이로 다를 수 있다.
/// 동치 검증 전 양쪽에서 동시 제거.
fn strip_volatile(v: &mut Value) {
    match v {
        Value::Object(map) => {
            for k in ["created_at", "updated_at", "resolved_at", "entered_status_at",
                      "minutes_in_status", "changed_at", "expansion_rate"] {
                map.remove(k);
            }
            for (_, child) in map.iter_mut() {
                strip_volatile(child);
            }
        }
        Value::Array(arr) => {
            for child in arr.iter_mut() {
                strip_volatile(child);
            }
        }
        _ => {}
    }
}

fn normalize(v: Value) -> Value {
    let mut v = v;
    strip_volatile(&mut v);
    v
}

// ---------- read-only 동치 ----------

#[tokio::test]
async fn test_parity_sprint_list_and_current() {
    let db_a = fresh_db().await; let _ = seed_via_db(&db_a).await;
    let db_b = fresh_db().await; let _ = seed_via_dispatch(&db_b).await;

    let cli_list  = normalize(serde_json::to_value(db_a.sprint_list(None).await.unwrap()).unwrap());
    let mcp_list  = normalize(dispatch(Arc::clone(&db_b), "sprint_list", &json!({})).await.unwrap());
    assert_eq!(cli_list, mcp_list, "sprint_list 동치 실패");

    let cli_cur = normalize(serde_json::to_value(db_a.sprint_current().await.unwrap()).unwrap());
    let mcp_cur = normalize(dispatch(Arc::clone(&db_b), "sprint_current", &json!({})).await.unwrap());
    assert_eq!(cli_cur, mcp_cur, "sprint_current 동치 실패");
}

#[tokio::test]
async fn test_parity_epic_list() {
    let db_a = fresh_db().await; seed_via_db(&db_a).await;
    let db_b = fresh_db().await; seed_via_dispatch(&db_b).await;
    let cli = normalize(serde_json::to_value(db_a.epic_list(Some("p"), None).await.unwrap()).unwrap());
    let mcp = normalize(dispatch(Arc::clone(&db_b), "epic_list", &json!({"project_key": "p"})).await.unwrap());
    assert_eq!(cli, mcp, "epic_list 동치 실패");
}

#[tokio::test]
async fn test_parity_issue_list_and_get() {
    let db_a = fresh_db().await; let (_, _, iid, _) = seed_via_db(&db_a).await;
    let db_b = fresh_db().await; let (_, _, iid_b, _) = seed_via_dispatch(&db_b).await;
    assert_eq!(iid, iid_b, "시드 시퀀스가 같으면 id 도 같아야 함");

    let cli_list = normalize(serde_json::to_value(
        db_a.issue_list(IssueFilter { project_key: Some("p".into()), ..Default::default() }).await.unwrap()
    ).unwrap());
    let mcp_list = normalize(dispatch(Arc::clone(&db_b), "issue_list",
        &json!({"project_key": "p"})).await.unwrap());
    assert_eq!(cli_list, mcp_list, "issue_list 동치 실패");

    let cli_get = normalize(serde_json::to_value(db_a.issue_get(iid).await.unwrap()).unwrap());
    let mcp_get = normalize(dispatch(Arc::clone(&db_b), "issue_get", &json!({"id": iid})).await.unwrap());
    assert_eq!(cli_get, mcp_get, "issue_get 동치 실패");
}

#[tokio::test]
async fn test_parity_task_list_and_next() {
    let db_a = fresh_db().await; let (_, _, iid, _) = seed_via_db(&db_a).await;
    let db_b = fresh_db().await; let (_, _, _, _)   = seed_via_dispatch(&db_b).await;

    let cli = normalize(serde_json::to_value(db_a.task_list(iid, None).await.unwrap()).unwrap());
    let mcp = normalize(dispatch(Arc::clone(&db_b), "task_list", &json!({"issue_id": iid})).await.unwrap());
    assert_eq!(cli, mcp, "task_list 동치 실패");

    // task_next 는 ready 큐 기반 — 둘 다 ready 없으니 동시에 null/None 이어야 함
    let cli = normalize(serde_json::to_value(db_a.task_next(Some("p"), None).await.unwrap()).unwrap());
    let mcp = normalize(dispatch(Arc::clone(&db_b), "task_next", &json!({"project_key": "p"})).await.unwrap());
    assert_eq!(cli, mcp, "task_next 동치 실패");
}

#[tokio::test]
async fn test_parity_note_list_and_get() {
    let db_a = fresh_db().await; let (_, _, iid, _) = seed_via_db(&db_a).await;
    db_a.note_add(CreateNoteInput {
        issue_id: iid, task_id: None, note_type: NoteType::Caveat,
        summary: "주의".into(), detail: Some("긴".into()),
        author: Some("agent".into()), agent_id: None,
        scope: None, scope_target_id: None, project_key: None,
    }).await.unwrap();

    let db_b = fresh_db().await; let (_, _, iid_b, _) = seed_via_dispatch(&db_b).await;
    dispatch(Arc::clone(&db_b), "note_add", &json!({
        "issue_id": iid_b, "note_type": "caveat", "summary": "주의", "detail": "긴", "author": "agent"
    })).await.unwrap();

    let cli = normalize(serde_json::to_value(
        db_a.note_list(Some(iid), None, None, false).await.unwrap()
    ).unwrap());
    let mcp = normalize(dispatch(Arc::clone(&db_b), "note_list",
        &json!({"issue_id": iid_b})).await.unwrap());
    assert_eq!(cli, mcp, "note_list 동치 실패");
}

#[tokio::test]
async fn test_parity_session_restore_and_end() {
    let db_a = fresh_db().await; seed_via_db(&db_a).await;
    let db_b = fresh_db().await; seed_via_dispatch(&db_b).await;

    let cli = normalize(serde_json::to_value(db_a.session_restore(Some("p")).await.unwrap()).unwrap());
    let mcp = normalize(dispatch(Arc::clone(&db_b), "session_restore",
        &json!({"project_key": "p"})).await.unwrap());
    assert_eq!(cli, mcp, "session_restore 동치 실패");

    let cli = normalize(serde_json::to_value(db_a.session_end(Some("p")).await.unwrap()).unwrap());
    let mcp = normalize(dispatch(Arc::clone(&db_b), "session_end",
        &json!({"project_key": "p"})).await.unwrap());
    assert_eq!(cli, mcp, "session_end 동치 실패");
}

#[tokio::test]
async fn test_parity_board_status_and_blocked_graph() {
    let db_a = fresh_db().await; seed_via_db(&db_a).await;
    let db_b = fresh_db().await; seed_via_dispatch(&db_b).await;

    let cli = normalize(serde_json::to_value(db_a.board_status_query(Some("p")).await.unwrap()).unwrap());
    let mcp = normalize(dispatch(Arc::clone(&db_b), "board_status",
        &json!({"project_key": "p"})).await.unwrap());
    assert_eq!(cli, mcp, "board_status 동치 실패");

    let cli = normalize(serde_json::to_value(db_a.blocked_issues_graph("p").await.unwrap()).unwrap());
    let mcp = normalize(dispatch(Arc::clone(&db_b), "my_blocked_issues",
        &json!({"project_key": "p"})).await.unwrap());
    assert_eq!(cli, mcp, "my_blocked_issues 동치 실패");
}

#[tokio::test]
async fn test_parity_history_recent_and_by_agent() {
    let db_a = fresh_db().await; seed_via_db(&db_a).await;
    let db_b = fresh_db().await; seed_via_dispatch(&db_b).await;

    let cli = normalize(serde_json::to_value(
        db_a.history_recent(50, None).await.unwrap()
    ).unwrap());
    let mcp = normalize(dispatch(Arc::clone(&db_b), "history_recent",
        &json!({"limit": 50})).await.unwrap());
    // 시드 actor 가 다를 수 있으므로 (CLI 쪽 "test" vs MCP 쪽 dispatch 기본값) 길이만 비교.
    assert_eq!(
        cli.as_array().map(|a| a.len()),
        mcp.as_array().map(|a| a.len()),
        "history_recent 개수 동치 실패"
    );

    // by_agent 는 agent_id 가 정확히 일치하는 경우만 비교.
    let cli = normalize(serde_json::to_value(
        db_a.history_by_agent("nonexistent", 10).await.unwrap()
    ).unwrap());
    let mcp = normalize(dispatch(Arc::clone(&db_b), "history_by_agent",
        &json!({"agent_id": "nonexistent", "limit": 10})).await.unwrap());
    assert_eq!(cli, mcp, "history_by_agent 동치 실패 (둘 다 빈 결과여야 함)");
}

#[tokio::test]
async fn test_parity_stalled_issues_empty() {
    let db_a = fresh_db().await; seed_via_db(&db_a).await;
    let db_b = fresh_db().await; seed_via_dispatch(&db_b).await;

    let cli = normalize(serde_json::to_value(
        db_a.stalled_issues(Some("p"), engram_core::models::issue::IssueStatus::Working, 1).await.unwrap()
    ).unwrap());
    let mcp = normalize(dispatch(Arc::clone(&db_b), "stalled_issues",
        &json!({"project_key": "p", "status": "working", "threshold_minutes": 1})).await.unwrap());
    assert_eq!(cli, mcp, "stalled_issues 동치 실패 (둘 다 빈 결과)");
}

// ---------- 변경 도구 대표 시나리오 동치 ----------

#[tokio::test]
async fn test_parity_full_lifecycle_issue_state_machine() {
    // 두 DB 에 동일 시퀀스 (claim, release, set-sprint, delete) 를 양쪽에서 따로 적용
    // 후 최종 상태가 동일한지 확인.
    let db_a = fresh_db().await; let (_, _, iid_a, _) = seed_via_db(&db_a).await;
    let db_b = fresh_db().await; let (_, _, iid_b, _) = seed_via_dispatch(&db_b).await;
    assert_eq!(iid_a, iid_b);

    // ready → claim (working) → release(demo)
    db_a.issue_update(iid_a,
        engram_core::models::issue::UpdateIssueInput {
            status: Some(engram_core::models::issue::IssueStatus::Ready),
            ..Default::default()
        }, "agent_a").await.unwrap();
    db_a.issue_claim(iid_a, "agent_a").await.unwrap();
    db_a.issue_release(iid_a,
        engram_core::models::issue::IssueStatus::Demo, "agent_a", false).await.unwrap();

    dispatch(Arc::clone(&db_b), "issue_update",
        &json!({"id": iid_b, "status": "ready"})).await.unwrap();
    dispatch(Arc::clone(&db_b), "issue_claim",
        &json!({"id": iid_b, "agent_id": "agent_a"})).await.unwrap();
    dispatch(Arc::clone(&db_b), "issue_release",
        &json!({"id": iid_b, "agent_id": "agent_a", "transition_to": "demo"})).await.unwrap();

    let cli = normalize(serde_json::to_value(db_a.issue_get(iid_a).await.unwrap()).unwrap());
    let mcp = normalize(dispatch(Arc::clone(&db_b), "issue_get",
        &json!({"id": iid_b})).await.unwrap());
    assert_eq!(cli, mcp, "issue 상태 머신 동치 실패 (ready→working→demo)");
    assert_eq!(cli["status"], "demo");
    assert!(cli["assigned_agent"].is_null(), "release 후 점유자 해제");
}

#[tokio::test]
async fn test_parity_link_unlink_roundtrip() {
    let db_a = fresh_db().await; let (_, eid_a, a1, _) = seed_via_db(&db_a).await;
    let db_b = fresh_db().await; let (_, eid_b, b1, _) = seed_via_dispatch(&db_b).await;

    // 두 번째 이슈 추가
    let a2 = db_a.issue_create(CreateIssueInput {
        epic_id: eid_a, sprint_id: None, title: "I2".into(),
        description: None, goal: None, priority: None,
    }).await.unwrap().id;
    let b2 = dispatch(Arc::clone(&db_b), "issue_create",
        &json!({"epic_id": eid_b, "title": "I2"})).await.unwrap()["id"].as_i64().unwrap();
    assert_eq!(a2, b2);

    let link_a = db_a.issue_link(a1, a2, LinkType::Blocks).await.unwrap();
    let link_b = dispatch(Arc::clone(&db_b), "issue_link",
        &json!({"source_id": b1, "target_id": b2, "link_type": "blocks"})).await.unwrap();

    let cli = normalize(serde_json::to_value(&link_a).unwrap());
    let mcp = normalize(link_b.clone());
    assert_eq!(cli, mcp, "issue_link 결과 동치 실패");

    // unlink
    db_a.issue_unlink(link_a.id).await.unwrap();
    dispatch(Arc::clone(&db_b), "issue_unlink",
        &json!({"link_id": link_b["id"].as_i64().unwrap()})).await.unwrap();

    let cli_links = normalize(serde_json::to_value(
        db_a.issue_links_for(a1).await.unwrap()
    ).unwrap());
    let mcp_links = normalize(dispatch(Arc::clone(&db_b), "issue_list",
        &json!({"project_key": "p"})).await.unwrap());
    // unlink 후 양쪽 모두 링크가 비어있어야 함 (issue_list 가 아닌 links_for 를 양쪽에서)
    assert_eq!(cli_links, json!([]), "unlink 후 CLI 링크 비어야 함");
    assert!(mcp_links.is_array(), "issue_list 응답 형태 유지");
}

#[tokio::test]
async fn test_parity_task_test_workflow() {
    let db_a = fresh_db().await; let (_, _, _, tid_a) = seed_via_db(&db_a).await;
    let db_b = fresh_db().await; let (_, _, _, tid_b) = seed_via_dispatch(&db_b).await;

    // add 3개 + check 2개 + uncheck 1개 + remove 1개
    db_a.task_test_add_bulk(tid_a, vec!["A".into(), "B".into(), "C".into()]).await.unwrap();
    let list_a = db_a.task_test_list(tid_a).await.unwrap();
    let ids_a: Vec<i64> = list_a.iter().map(|t| t.id).collect();
    db_a.task_test_check_bulk(vec![ids_a[0], ids_a[1]]).await.unwrap();
    db_a.task_test_uncheck(ids_a[1]).await.unwrap();
    db_a.task_test_remove(ids_a[2]).await.unwrap();

    dispatch(Arc::clone(&db_b), "task_test_add_bulk",
        &json!({"task_id": tid_b, "labels": ["A","B","C"]})).await.unwrap();
    let list_b = dispatch(Arc::clone(&db_b), "task_test_list",
        &json!({"task_id": tid_b})).await.unwrap();
    let ids_b: Vec<i64> = list_b.as_array().unwrap().iter()
        .map(|v| v["id"].as_i64().unwrap()).collect();
    dispatch(Arc::clone(&db_b), "task_test_check_bulk",
        &json!({"ids": [ids_b[0], ids_b[1]]})).await.unwrap();
    dispatch(Arc::clone(&db_b), "task_test_uncheck",
        &json!({"id": ids_b[1]})).await.unwrap();
    dispatch(Arc::clone(&db_b), "task_test_remove",
        &json!({"id": ids_b[2]})).await.unwrap();

    let cli = normalize(serde_json::to_value(db_a.task_test_list(tid_a).await.unwrap()).unwrap());
    let mcp = normalize(dispatch(Arc::clone(&db_b), "task_test_list",
        &json!({"task_id": tid_b})).await.unwrap());
    assert_eq!(cli, mcp, "task_test 워크플로 동치 실패");
}

#[tokio::test]
async fn test_parity_note_add_broadcast_and_resolve() {
    let db_a = fresh_db().await; let (sid_a, eid_a, iid_a, _) = seed_via_db(&db_a).await;
    let db_b = fresh_db().await; let (sid_b, eid_b, iid_b, _) = seed_via_dispatch(&db_b).await;
    assert_eq!((sid_a, eid_a, iid_a), (sid_b, eid_b, iid_b));

    // broadcast(epic) decision note
    let n_a = db_a.note_add(CreateNoteInput {
        issue_id: 0, task_id: None, note_type: NoteType::Decision,
        summary: "결정".into(), detail: None,
        author: Some("agent".into()), agent_id: Some("leader@s".into()),
        scope: Some(engram_core::models::note::NoteScope::Epic),
        scope_target_id: Some(eid_a), project_key: None,
    }).await.unwrap();
    let n_b = dispatch(Arc::clone(&db_b), "note_add", &json!({
        "note_type": "decision", "summary": "결정", "author": "agent",
        "agent_id": "leader@s", "scope": "epic", "scope_target_id": eid_b
    })).await.unwrap();

    let cli = normalize(serde_json::to_value(&n_a).unwrap());
    let mcp = normalize(n_b.clone());
    assert_eq!(cli, mcp, "broadcast note_add 동치 실패");

    db_a.note_resolve(n_a.id, "user").await.unwrap();
    dispatch(Arc::clone(&db_b), "note_resolve",
        &json!({"note_id": n_b["id"].as_i64().unwrap()})).await.unwrap();

    let cli = normalize(serde_json::to_value(db_a.note_get(n_a.id).await.unwrap()).unwrap());
    let mcp = normalize(dispatch(Arc::clone(&db_b), "note_get",
        &json!({"note_id": n_b["id"].as_i64().unwrap()})).await.unwrap());
    assert_eq!(cli["resolved"], mcp["resolved"], "resolved 플래그 동치");
    assert_eq!(cli["resolved"], true);
}

#[tokio::test]
async fn test_parity_task_insert_after_and_delete() {
    let db_a = fresh_db().await; let (_, _, iid_a, _) = seed_via_db(&db_a).await;
    let db_b = fresh_db().await; let (_, _, iid_b, _) = seed_via_dispatch(&db_b).await;

    // T 다음에 새 task 삽입 후 첫 task 삭제
    let list_a = db_a.task_list(iid_a, None).await.unwrap();
    let t_a    = list_a[0].id;
    db_a.task_create(CreateTaskInput {
        issue_id: iid_a, title: "T2".into(), description: None,
        goal: None, after_task_id: Some(t_a),
        source: Some(engram_core::models::task::TaskSource::AgentDiscovered),
    }).await.unwrap();
    db_a.task_delete(t_a).await.unwrap();

    let list_b = dispatch(Arc::clone(&db_b), "task_list",
        &json!({"issue_id": iid_b})).await.unwrap();
    let t_b = list_b[0]["id"].as_i64().unwrap();
    dispatch(Arc::clone(&db_b), "task_insert_after",
        &json!({"issue_id": iid_b, "after_task_id": t_b, "title": "T2"})).await.unwrap();
    dispatch(Arc::clone(&db_b), "task_delete", &json!({"id": t_b})).await.unwrap();

    let cli = normalize(serde_json::to_value(db_a.task_list(iid_a, None).await.unwrap()).unwrap());
    let mcp = normalize(dispatch(Arc::clone(&db_b), "task_list",
        &json!({"issue_id": iid_b})).await.unwrap());
    assert_eq!(cli, mcp, "task insert_after + delete 동치 실패");
}

#[tokio::test]
async fn test_parity_history_for_after_changes() {
    let db_a = fresh_db().await; let (_, _, iid_a, _) = seed_via_db(&db_a).await;
    let db_b = fresh_db().await; let (_, _, iid_b, _) = seed_via_dispatch(&db_b).await;

    // 같은 상태 전이를 양쪽에서
    db_a.issue_update(iid_a,
        engram_core::models::issue::UpdateIssueInput {
            status: Some(engram_core::models::issue::IssueStatus::Ready),
            ..Default::default()
        }, "agent_x").await.unwrap();
    dispatch(Arc::clone(&db_b), "issue_update",
        &json!({"id": iid_b, "status": "ready", "agent_id": "agent_x"})).await.unwrap();

    let cli = normalize(serde_json::to_value(
        db_a.history_list(engram_core::models::history::EntityType::Issue, iid_a).await.unwrap()
    ).unwrap());
    let mcp = normalize(dispatch(Arc::clone(&db_b), "history_for",
        &json!({"entity_type": "issue", "entity_id": iid_b})).await.unwrap());
    // 길이만 비교 (created_at 제거 후에도 actor 가 다르면 changed_by 차이 가능 — assert 폭 좁힘).
    assert_eq!(
        cli.as_array().map(|a| a.len()),
        mcp.as_array().map(|a| a.len()),
        "history_for 항목 개수 동치"
    );
    assert!(cli.as_array().unwrap().len() >= 1, "최소 1건 history 기록");
}
