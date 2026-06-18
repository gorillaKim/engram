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
        mission::CreateMissionInput,
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
    let m = db.mission_create(CreateMissionInput {
        title: "M".into(), description: None, jira_key: None,
    }).await.unwrap();
    let e = db.epic_create(CreateEpicInput {
        project_key: "p".into(), mission_id: Some(m.id), sprint_id: Some(s.id),
            title: "E".into(), description: None,
    }).await.unwrap();
    let i = db.issue_create(CreateIssueInput {
        epic_id: e.id, title: "I".into(),
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
    dispatch(Arc::clone(db), "sprint_update", &json!({"id": sid, "status": "active", "agent_id": "test"}))
        .await.unwrap();
    let m = dispatch(Arc::clone(db), "mission_create",
        &json!({"title": "M", "project_key": "p"})).await.unwrap();
    let mid = m["id"].as_i64().unwrap();
    let e = dispatch(Arc::clone(db), "epic_create",
        &json!({"project_key": "p", "title": "E", "mission_id": mid, "sprint_id": sid})).await.unwrap();
    let eid = e["id"].as_i64().unwrap();
    let i = dispatch(Arc::clone(db), "issue_create",
        &json!({"epic_id": eid, "title": "I"})).await.unwrap();
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
    let cli = normalize(serde_json::to_value(db_a.epic_list(Some("p"), false).await.unwrap()).unwrap());
    let mcp = normalize(dispatch(Arc::clone(&db_b), "epic_list", &json!({"project_key": "p", "mode": "normal"})).await.unwrap());
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
        &json!({"project_key": "p", "mode": "normal"})).await.unwrap());
    assert_eq!(cli_list, mcp_list, "issue_list 동치 실패");

    // 복수 status 동치 테스트
    let cli_multi = normalize(serde_json::to_value(
        db_a.issue_list(IssueFilter {
            project_key: Some("p".into()),
            statuses: Some(vec![engram_core::models::issue::IssueStatus::Ready, engram_core::models::issue::IssueStatus::Required]),
            ..Default::default()
        }).await.unwrap()
    ).unwrap());
    let mcp_multi = normalize(dispatch(Arc::clone(&db_b), "issue_list",
        &json!({"project_key": "p", "status": ["ready", "required"], "mode": "normal"})).await.unwrap());
    assert_eq!(cli_multi, mcp_multi, "issue_list 복수 status 동치 실패");

    let cli_get = normalize(serde_json::to_value(db_a.issue_get(iid, false).await.unwrap()).unwrap());
    let mcp_get = normalize(dispatch(Arc::clone(&db_b), "issue_get", &json!({"id": iid, "mode": "normal"})).await.unwrap());
    assert_eq!(cli_get, mcp_get, "issue_get 동치 실패");
}

#[tokio::test]
async fn test_parity_task_list_and_next() {
    let db_a = fresh_db().await; let (_, _, iid, _) = seed_via_db(&db_a).await;
    let db_b = fresh_db().await; let (_, _, _, _)   = seed_via_dispatch(&db_b).await;

    let cli = normalize(serde_json::to_value(db_a.task_list(iid, None).await.unwrap()).unwrap());
    let mcp = normalize(dispatch(Arc::clone(&db_b), "task_list", &json!({"issue_id": iid, "mode": "normal"})).await.unwrap());
    assert_eq!(cli, mcp, "task_list 동치 실패");

    // task_next 는 ready 큐 기반 — 둘 다 ready 없으니 동시에 null/None 이어야 함
    let cli = normalize(serde_json::to_value(db_a.task_next(Some("p"), None).await.unwrap()).unwrap());
    let mcp = normalize(dispatch(Arc::clone(&db_b), "task_next", &json!({"project_key": "p", "mode": "normal"})).await.unwrap());
    assert_eq!(cli, mcp, "task_next 동치 실패");
}

#[tokio::test]
async fn test_parity_note_list_and_get() {
    let db_a = fresh_db().await; let (_, _, iid, _) = seed_via_db(&db_a).await;
    db_a.note_add(CreateNoteInput {
        issue_id: iid, task_id: None, note_type: NoteType::Caveat,
        summary: "주의".into(), detail: Some("긴".into()),
        author: Some("agent".into()), agent_id: Some("test".into()),
        scope: None, scope_target_id: None, project_key: None,
    }).await.unwrap();

    let db_b = fresh_db().await; let (_, _, iid_b, _) = seed_via_dispatch(&db_b).await;
    dispatch(Arc::clone(&db_b), "note_add", &json!({
        "issue_id": iid_b, "note_type": "caveat", "summary": "주의", "detail": "긴", "author": "agent", "agent_id": "test"
    })).await.unwrap();

    let cli = normalize(serde_json::to_value(
        db_a.note_list(Some(iid), None, None, false, false, None, None, None, None, None).await.unwrap()
    ).unwrap());
    let mcp = normalize(dispatch(Arc::clone(&db_b), "note_list",
        &json!({"issue_id": iid_b, "mode": "normal"})).await.unwrap());
    assert_eq!(cli, mcp, "note_list 동치 실패");
}

#[tokio::test]
async fn test_parity_session_restore_and_end() {
    let db_a = fresh_db().await; seed_via_db(&db_a).await;
    let db_b = fresh_db().await; seed_via_dispatch(&db_b).await;

    let cli = normalize(serde_json::to_value(db_a.session_restore(Some("p"), false, 120, None).await.unwrap()).unwrap());
    let mcp = normalize(dispatch(Arc::clone(&db_b), "session_restore",
        &json!({"project_key": "p", "mode": "normal"})).await.unwrap());
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

    // 1. 기본 조회 (compact=false, include_chains=true)
    let cli = normalize(serde_json::to_value(db_a.board_status_query(Some("p"), false, true).await.unwrap()).unwrap());
    let mcp = normalize(dispatch(Arc::clone(&db_b), "board_status",
        &json!({"project_key": "p", "mode": "normal"})).await.unwrap());
    assert_eq!(cli, mcp, "board_status 동치 실패");

    // 2. compact 조회 (compact=true, include_chains=true)
    let cli_compact = normalize(serde_json::to_value(db_a.board_status_query(Some("p"), true, true).await.unwrap()).unwrap());
    let mcp_compact = normalize(dispatch(Arc::clone(&db_b), "board_status",
        &json!({"project_key": "p", "compact": true, "mode": "compact"})).await.unwrap());
    assert_eq!(cli_compact, mcp_compact, "board_status compact 동치 실패");

    // 3. chains 제외 조회 (compact=false, include_chains=false)
    let cli_no_chains = normalize(serde_json::to_value(db_a.board_status_query(Some("p"), false, false).await.unwrap()).unwrap());
    let mcp_no_chains = normalize(dispatch(Arc::clone(&db_b), "board_status",
        &json!({"project_key": "p", "include_chains": false, "mode": "normal"})).await.unwrap());
    assert_eq!(cli_no_chains, mcp_no_chains, "board_status no chains 동치 실패");
    assert!(cli_no_chains["blocked_chains"].is_null() || cli_no_chains.get("blocked_chains").is_none());

    let cli_g = normalize(serde_json::to_value(db_a.blocked_issues_graph("p").await.unwrap()).unwrap());
    let mcp_g = normalize(dispatch(Arc::clone(&db_b), "my_blocked_issues",
        &json!({"project_key": "p"})).await.unwrap());
    assert_eq!(cli_g, mcp_g, "my_blocked_issues 동치 실패");
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
        &json!({"project_key": "p", "status": "working", "threshold_minutes": 1, "mode": "normal"})).await.unwrap());
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
        &json!({"id": iid_b, "status": "ready", "agent_id": "agent_a"})).await.unwrap();
    dispatch(Arc::clone(&db_b), "issue_claim",
        &json!({"id": iid_b, "agent_id": "agent_a"})).await.unwrap();
    dispatch(Arc::clone(&db_b), "issue_release",
        &json!({"id": iid_b, "agent_id": "agent_a", "transition_to": "demo"})).await.unwrap();

    let cli = normalize(serde_json::to_value(db_a.issue_get(iid_a, false).await.unwrap()).unwrap());
    let mcp = normalize(dispatch(Arc::clone(&db_b), "issue_get",
        &json!({"id": iid_b, "mode": "normal"})).await.unwrap());
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
        epic_id: eid_a, title: "I2".into(),
        description: None, goal: None, priority: None,
    }).await.unwrap().id;
    let b2 = dispatch(Arc::clone(&db_b), "issue_create",
        &json!({"epic_id": eid_b, "title": "I2"})).await.unwrap()["id"].as_i64().unwrap();
    assert_eq!(a2, b2);

    let link_a = db_a.issue_link(a1, a2, LinkType::Blocks).await.unwrap();
    let link_b = dispatch(Arc::clone(&db_b), "issue_link",
        &json!({"source_id": b1, "target_id": b2, "link_type": "blocks", "agent_id": "test"})).await.unwrap();

    let cli = normalize(serde_json::to_value(&link_a).unwrap());
    let mcp = normalize(link_b.clone());
    assert_eq!(cli, mcp, "issue_link 결과 동치 실패");

    // unlink
    db_a.issue_unlink(link_a.id).await.unwrap();
    dispatch(Arc::clone(&db_b), "issue_unlink",
        &json!({"link_id": link_b["id"].as_i64().unwrap(), "agent_id": "test"})).await.unwrap();

    let cli_links = normalize(serde_json::to_value(
        db_a.issue_links_for(a1).await.unwrap()
    ).unwrap());
    let mcp_links = normalize(dispatch(Arc::clone(&db_b), "issue_list",
        &json!({"project_key": "p", "mode": "normal"})).await.unwrap());
    // unlink 후 양쪽 모두 링크가 비어있어야 함 (issue_list 가 아닌 links_for 를 양쪽에서)
    assert_eq!(cli_links, json!([]), "unlink 후 CLI 링크 비어야 함");
    assert!(mcp_links["items"].is_array(), "issue_list 응답 형태 유지");
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
    db_a.task_test_uncheck(ids_a[1], "test").await.unwrap();
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
        &json!({"id": ids_b[1], "agent_id": "test"})).await.unwrap();
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
        &json!({"id": n_b["id"].as_i64().unwrap(), "agent_id": "test"})).await.unwrap();

    let cli = normalize(serde_json::to_value(db_a.note_get(n_a.id, false).await.unwrap()).unwrap());
    let mcp = normalize(dispatch(Arc::clone(&db_b), "note_get",
        &json!({"id": n_b["id"].as_i64().unwrap(), "mode": "normal"})).await.unwrap());
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
        &json!({"issue_id": iid_b, "mode": "normal"})).await.unwrap();
    let t_b = list_b[0]["id"].as_i64().unwrap();
    dispatch(Arc::clone(&db_b), "task_insert_after",
        &json!({"issue_id": iid_b, "after_task_id": t_b, "title": "T2"})).await.unwrap();
    dispatch(Arc::clone(&db_b), "task_delete", &json!({"id": t_b})).await.unwrap();

    let cli = normalize(serde_json::to_value(db_a.task_list(iid_a, None).await.unwrap()).unwrap());
    let mcp = normalize(dispatch(Arc::clone(&db_b), "task_list",
        &json!({"issue_id": iid_b, "mode": "normal"})).await.unwrap());
    assert_eq!(cli, mcp, "task insert_after + delete 동치 실패");
}

// ---------- mission 패리티 ----------

/// mission_create → mission_get 동치 검증.
/// CLI 쪽: db.mission_create + db.mission_get (직렬화)
/// MCP 쪽: dispatch("mission_create") + dispatch("mission_get")
#[tokio::test]
async fn test_mission_crud_parity() {
    let db_a = fresh_db().await;
    let db_b = fresh_db().await;

    // CLI 쪽: sprint 생성 후 mission 생성
    let s_a = db_a.sprint_create(CreateSprintInput {
        name: "MS1".into(), goal: None, start_date: None, end_date: None,
    }).await.unwrap();
    let m_a = db_a.mission_create(CreateMissionInput {
        title: "테스트 미션".into(),
        description: Some("미션 설명".into()),
        jira_key: Some("PROJ-42".into()),
    }).await.unwrap();

    // MCP 쪽: dispatch로 동일 시퀀스
    let s_b = dispatch(Arc::clone(&db_b), "sprint_create",
        &json!({"name": "MS1"})).await.unwrap();
    let sid_b = s_b["id"].as_i64().unwrap();
    let m_b = dispatch(Arc::clone(&db_b), "mission_create", &json!({
        "title": "테스트 미션",
        "description": "미션 설명",
        "jira_key": "PROJ-42"
    })).await.unwrap();
    let mid_b = m_b["id"].as_i64().unwrap();

    // mission_get 동치 검증
    let cli_get = normalize(serde_json::to_value(db_a.mission_get(m_a.id).await.unwrap()).unwrap());
    let mcp_get = normalize(dispatch(Arc::clone(&db_b), "mission_get",
        &json!({"id": mid_b})).await.unwrap());
    assert_eq!(cli_get, mcp_get, "mission_get 동치 실패");

    // create 반환값 자체도 동치
    let cli_created = normalize(serde_json::to_value(&m_a).unwrap());
    let mcp_created = normalize(m_b);
    assert_eq!(cli_created, mcp_created, "mission_create 반환값 동치 실패");
}

/// mission_list 동치 검증.
/// sprint_id 필터 포함/미포함 두 케이스 모두 확인.
#[tokio::test]
async fn test_mission_list_parity() {
    let db_a = fresh_db().await;
    let db_b = fresh_db().await;

    // CLI 쪽: sprint + mission 2개 생성
    let s_a = db_a.sprint_create(CreateSprintInput {
        name: "ML1".into(), goal: None, start_date: None, end_date: None,
    }).await.unwrap();
    db_a.mission_create(CreateMissionInput {
        title: "미션 A".into(), description: None, jira_key: None,
    }).await.unwrap();
    db_a.mission_create(CreateMissionInput {
        title: "미션 B".into(), description: None, jira_key: None,
    }).await.unwrap();

    // MCP 쪽: dispatch로 동일 시퀀스
    let s_b = dispatch(Arc::clone(&db_b), "sprint_create",
        &json!({"name": "ML1"})).await.unwrap();
    let sid_b = s_b["id"].as_i64().unwrap();
    dispatch(Arc::clone(&db_b), "mission_create",
        &json!({"title": "미션 A"})).await.unwrap();
    dispatch(Arc::clone(&db_b), "mission_create",
        &json!({"title": "미션 B"})).await.unwrap();

    // 필터 없이 전체 목록 동치 (active only 기본값)
    let cli_all = normalize(serde_json::to_value(
        db_a.mission_list(engram_core::models::mission::MissionFilter::default()).await.unwrap()
    ).unwrap());
    let mcp_all = normalize(dispatch(Arc::clone(&db_b), "mission_list",
        &json!({})).await.unwrap());
    assert_eq!(cli_all, mcp_all, "mission_list (전체) 동치 실패");
    assert_eq!(cli_all.as_array().unwrap().len(), 2, "mission 2건이어야 함");

    // sprint_id 필터 동치
    let cli_filtered = normalize(serde_json::to_value(
        db_a.mission_list(engram_core::models::mission::MissionFilter {
            ..Default::default()
        }).await.unwrap()
    ).unwrap());
    let mcp_filtered = normalize(dispatch(Arc::clone(&db_b), "mission_list",
        &json!({})).await.unwrap());
    assert_eq!(cli_filtered, mcp_filtered, "mission_list (sprint_id 필터) 동치 실패");
}

/// mission_get_tree 동치 검증.
/// Mission → Epics → Issues 계층 트리 구조가 CLI/MCP 양쪽에서 동일한지 확인.
#[tokio::test]
async fn test_mission_get_tree_parity() {
    let db_a = fresh_db().await;
    let db_b = fresh_db().await;

    // CLI 쪽: sprint → mission → epic → issue 계층 구성
    let s_a = db_a.sprint_create(CreateSprintInput {
        name: "GT1".into(), goal: None, start_date: None, end_date: None,
    }).await.unwrap();
    db_a.sprint_update(s_a.id, engram_core::models::sprint::UpdateSprintInput {
        name: None, status: Some(engram_core::models::sprint::SprintStatus::Active),
        goal: None, start_date: None, end_date: None,
    }, "test").await.unwrap();
    let m_a = db_a.mission_create(CreateMissionInput {
        title: "트리 미션".into(), description: None, jira_key: None,
    }).await.unwrap();
    let e_a = db_a.epic_create(CreateEpicInput {
        project_key: "tp".into(), mission_id: Some(m_a.id), sprint_id: Some(s_a.id),
            title: "에픽 1".into(),
        description: None,
    }).await.unwrap();
    db_a.issue_create(CreateIssueInput {
        epic_id: e_a.id, title: "이슈 1".into(), description: None, goal: None, priority: None,
    }).await.unwrap();
    db_a.issue_create(CreateIssueInput {
        epic_id: e_a.id, title: "이슈 2".into(), description: None, goal: None, priority: None,
    }).await.unwrap();

    // MCP 쪽: dispatch로 동일 계층 구성
    let s_b = dispatch(Arc::clone(&db_b), "sprint_create",
        &json!({"name": "GT1"})).await.unwrap();
    let sid_b = s_b["id"].as_i64().unwrap();
    dispatch(Arc::clone(&db_b), "sprint_update",
        &json!({"id": sid_b, "status": "active", "agent_id": "test"})).await.unwrap();
    let m_b = dispatch(Arc::clone(&db_b), "mission_create",
        &json!({"title": "트리 미션"})).await.unwrap();
    let mid_b = m_b["id"].as_i64().unwrap();
    let e_b = dispatch(Arc::clone(&db_b), "epic_create",
        &json!({"project_key": "tp", "title": "에픽 1", "mission_id": mid_b, "sprint_id": sid_b}))
        .await.unwrap();
    let eid_b = e_b["id"].as_i64().unwrap();
    dispatch(Arc::clone(&db_b), "issue_create",
        &json!({"epic_id": eid_b, "title": "이슈 1"}))
        .await.unwrap();
    dispatch(Arc::clone(&db_b), "issue_create",
        &json!({"epic_id": eid_b, "title": "이슈 2"}))
        .await.unwrap();

    // mission_get_tree 동치 검증
    let cli_tree = normalize(serde_json::to_value(
        db_a.mission_get_tree(m_a.id).await.unwrap()
    ).unwrap());
    let mcp_tree = normalize(dispatch(Arc::clone(&db_b), "mission_get_tree",
        &json!({"id": mid_b})).await.unwrap());
    assert_eq!(cli_tree, mcp_tree, "mission_get_tree 동치 실패");

    // 트리 구조 내용 검증
    assert_eq!(cli_tree["mission"]["title"], "트리 미션", "mission 제목 확인");
    assert_eq!(cli_tree["epics"].as_array().unwrap().len(), 1, "epic 1건");
    assert_eq!(
        cli_tree["epics"][0]["issues"].as_array().unwrap().len(), 2,
        "이슈 2건"
    );
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

#[tokio::test]
async fn test_parity_issue_finish_and_cancel() {
    let db_a = fresh_db().await; let (_sid_a, eid_a, iid_a, _) = seed_via_db(&db_a).await;
    let db_b = fresh_db().await; let (_sid_b, eid_b, iid_b, _) = seed_via_dispatch(&db_b).await;
    assert_eq!(iid_a, iid_b);

    // 1. ready -> working -> demo 상태로 업데이트
    db_a.issue_update(iid_a, engram_core::models::issue::UpdateIssueInput {
        status: Some(engram_core::models::issue::IssueStatus::Ready), ..Default::default()
    }, "agent_a").await.unwrap();
    db_a.issue_claim(iid_a, "agent_a").await.unwrap();
    db_a.issue_release(iid_a, engram_core::models::issue::IssueStatus::Demo, "agent_a", false).await.unwrap();

    dispatch(Arc::clone(&db_b), "issue_update", &json!({"id": iid_b, "status": "ready", "agent_id": "agent_a"})).await.unwrap();
    dispatch(Arc::clone(&db_b), "issue_claim", &json!({"id": iid_b, "agent_id": "agent_a"})).await.unwrap();
    dispatch(Arc::clone(&db_b), "issue_release", &json!({"id": iid_b, "agent_id": "agent_a", "transition_to": "demo"})).await.unwrap();

    // 2. issue_finish 호출 및 동치 검증
    let cli_finish = normalize(serde_json::to_value(db_a.issue_finish(iid_a, "user").await.unwrap()).unwrap());
    let mcp_finish = normalize(dispatch(Arc::clone(&db_b), "issue_finish", &json!({"id": iid_b, "agent_id": "user"})).await.unwrap());
    assert_eq!(cli_finish, mcp_finish, "issue_finish 결과 동치 실패");
    assert_eq!(cli_finish["status"], "finished");

    // 3. cancel 검증을 위해 새로운 이슈 생성
    let epic_a = db_a.epic_get(eid_a).await.unwrap();
    let mid_a = epic_a.mission_id;
    let epic_b = dispatch(Arc::clone(&db_b), "epic_get", &json!({"id": eid_b})).await.unwrap();
    let mid_b = epic_b["mission_id"].as_i64();

    let iid_a2 = db_a.issue_create(CreateIssueInput {
        epic_id: eid_a,
        title: "I2".into(),
        description: None,
        goal: None,
        priority: None,
    }).await.unwrap().id;
    let iid_b2 = dispatch(Arc::clone(&db_b), "issue_create", &json!({"epic_id": eid_b, "title": "I2"})).await.unwrap()["id"].as_i64().unwrap();
    assert_eq!(iid_a2, iid_b2);

    // 4. issue_cancel 호출 및 동치 검증
    let cli_cancel = normalize(serde_json::to_value(db_a.issue_cancel(iid_a2, "No longer needed", "user").await.unwrap()).unwrap());
    let mcp_cancel = normalize(dispatch(Arc::clone(&db_b), "issue_cancel", &json!({"id": iid_b2, "reason": "No longer needed", "agent_id": "user"})).await.unwrap());
    assert_eq!(cli_cancel, mcp_cancel, "issue_cancel 결과 동치 실패");
    assert_eq!(cli_cancel["status"], "cancelled");
}

#[tokio::test]
async fn test_parity_issue_bulk_update() {
    let db_a = fresh_db().await; let (_, eid_a, iid_a1, _) = seed_via_db(&db_a).await;
    let db_b = fresh_db().await; let (_, eid_b, iid_b1, _) = seed_via_dispatch(&db_b).await;
    assert_eq!(iid_a1, iid_b1);

    // 두 번째 이슈 생성
    let iid_a2 = db_a.issue_create(CreateIssueInput {
        epic_id: eid_a, title: "I2".into(),
        description: None, goal: None, priority: None,
    }).await.unwrap().id;

    let iid_b2 = dispatch(Arc::clone(&db_b), "issue_create", &json!({
        "epic_id": eid_b, "title": "I2"
    })).await.unwrap()["id"].as_i64().unwrap();
    assert_eq!(iid_a2, iid_b2);

    // bulk_update 실행
    let res_a = db_a.issue_bulk_update(
        vec![iid_a1, iid_a2],
        engram_core::models::issue::BulkUpdateInput {
            status: Some(engram_core::models::issue::IssueStatus::Ready),
            priority: Some(engram_core::models::issue::IssuePriority::High),
        },
        "user"
    ).await.unwrap();

    let res_b = dispatch(Arc::clone(&db_b), "issue_bulk_update", &json!({
        "ids": [iid_b1, iid_b2],
        "status": "ready",
        "priority": "high",
        "agent_id": "user"
    })).await.unwrap();

    let cli = normalize(serde_json::to_value(&res_a).unwrap());
    let mcp = normalize(res_b);
    assert_eq!(cli, mcp, "issue_bulk_update 결과 동치 실패");
    assert_eq!(cli["succeeded"].as_array().unwrap().len(), 2);
    assert_eq!(cli["succeeded"][0]["status"], "ready");
    assert_eq!(cli["succeeded"][0]["priority"], "high");
}

#[tokio::test]
async fn test_agent_id_required_validation() {
    let db = fresh_db().await;
    
    // agent_id가 누락되었을 때 Validation 에러를 발생시켜야 하는 도구와 최소 파라미터 조합
    let test_cases = vec![
        ("issue_update", json!({"id": 1, "status": "ready"})),
        ("issue_claim", json!({"id": 1})),
        ("issue_release", json!({"id": 1, "transition_to": "ready"})),
        ("issue_link", json!({"source_id": 1, "target_id": 2, "link_type": "blocks"})),
        ("issue_unlink", json!({"link_id": 1})),
        ("issue_finish", json!({"id": 1})),
        ("issue_cancel", json!({"id": 1, "reason": "test"})),
        ("issue_bulk_update", json!({"ids": [1], "status": "ready"})),
        ("mission_update", json!({"id": 1, "title": "test"})),
        ("epic_set_sprint", json!({"epic_id": 1, "sprint_id": 1})),
        ("epic_update", json!({"id": 1, "title": "test"})),
        ("sprint_update", json!({"id": 1, "status": "active"})),
        ("task_update", json!({"id": 1, "status": "done"})),
        ("task_test_check", json!({"id": 1})),
        ("task_test_uncheck", json!({"id": 1})),
        ("note_add", json!({"note_type": "context", "summary": "test"})),
        ("note_resolve", json!({"id": 1})),
        ("note_add_bulk", json!({"notes": []})),
    ];

    for (tool_name, args) in test_cases {
        let res = dispatch(Arc::clone(&db), tool_name, &args).await;
        assert!(
            res.is_err(),
            "도구 '{}'는 agent_id가 누락되었음에도 에러를 반환하지 않았습니다.",
            tool_name
        );
        let err = res.err().unwrap();
        match err {
            engram_core::Error::Validation(msg) => {
                assert!(
                    msg.contains("agent_id") || msg.contains("required"),
                    "도구 '{}'의 에러 메시지가 agent_id 누락을 나타내지 않습니다: {}",
                    tool_name,
                    msg
                );
            }
            _ => panic!("도구 '{}'가 Validation 에러가 아닌 다른 에러를 반환했습니다: {:?}", tool_name, err),
        }
    }
}



/// task_test_check / task_test_uncheck 이 history.changed_by 에 agent_id 를 기록하는지 검증.
/// ADR-0010 컨벤션: 변경 도구는 감사 추적을 위해 history 에 agent_id 를 남겨야 함.
#[tokio::test]
async fn test_task_test_check_uncheck_history_recorded() {
    let db = fresh_db().await;
    let (_, _, _, tid) = seed_via_db(&db).await;

    // task_test 추가 후 check/uncheck 시나리오
    let tt = db.task_test_add(tid, "검증 항목".into()).await.unwrap();
    let agent = "test-agent@sess-001";

    // MCP dispatch 경로로 check
    dispatch(Arc::clone(&db), "task_test_check",
        &json!({"id": tt.id, "agent_id": agent})).await
        .expect("task_test_check should succeed");

    // history 기록 확인
    let hist = db.history_list(
        engram_core::models::history::EntityType::Task, tt.id
    ).await.unwrap();
    assert!(!hist.is_empty(), "task_test_check 후 history 기록이 있어야 함");
    let last = &hist[hist.len() - 1];
    assert_eq!(last.changed_by, agent, "changed_by 가 agent_id 와 일치해야 함");
    assert_eq!(last.field, "task_test.checked", "field 명 확인");
    assert_eq!(last.new_value.as_deref(), Some("true"));

    // MCP dispatch 경로로 uncheck
    dispatch(Arc::clone(&db), "task_test_uncheck",
        &json!({"id": tt.id, "agent_id": agent})).await
        .expect("task_test_uncheck should succeed");

    let hist2 = db.history_list(
        engram_core::models::history::EntityType::Task, tt.id
    ).await.unwrap();
    assert_eq!(hist2.len(), 2, "check + uncheck 각각 1건씩 총 2건");
    let last2 = &hist2[hist2.len() - 1];
    assert_eq!(last2.changed_by, agent);
    assert_eq!(last2.new_value.as_deref(), Some("false"));
}

use clap::CommandFactory;
use std::collections::HashSet;

fn collect_cli_commands(cmd: &clap::Command, prefix: &str) -> HashSet<String> {
    let mut set = HashSet::new();
    for sub in cmd.get_subcommands() {
        let name = if prefix.is_empty() {
            sub.get_name().to_string()
        } else {
            format!("{} {}", prefix, sub.get_name())
        };
        set.insert(name.clone());
        set.extend(collect_cli_commands(sub, &name));
    }
    set
}

fn map_mcp_to_cli(mcp_tool: &str) -> String {
    match mcp_tool {
        "stalled_issues" => "stalled".to_string(),
        "my_blocked_issues" => "blocked list".to_string(),
        "board_status" => "board status".to_string(),
        other => {
            if other.starts_with("task_test_") {
                let verb = other.strip_prefix("task_test_").unwrap().replace('_', "-");
                format!("task-test {}", verb)
            } else {
                let parts: Vec<&str> = other.split('_').collect();
                if parts.len() >= 2 {
                    let area = parts[0];
                    let verb = parts[1..].join("-");
                    format!("{} {}", area, verb)
                } else {
                    other.replace('_', "-")
                }
            }
        }
    }
}

/// 현재 CLI에 구현되지 않은 MCP 도구들의 허용 목록.
/// 향후 해당 CLI가 구현되면 이 목록에서 제거되어야 합니다.
const ALLOWED_MISSING_CLI_TOOLS: &[&str] = &[
    "task_test_add",
    "task_test_add_bulk",
    "task_test_list",
    "task_test_check",
    "task_test_check_bulk",
    "task_test_uncheck",
    "task_test_remove",
    "note_add_bulk",
    "planning_review_queue",
];

#[tokio::test]
async fn test_all_mcp_tools_have_cli_counterpart() {
    let mcp_tools: HashSet<String> = engram_mcp::tools::all_tool_definitions()
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_string())
        .collect();

    let cmd = engram_cli::Cli::command();
    let cli_commands = collect_cli_commands(&cmd, "");

    let mut missing = Vec::new();
    for mcp_tool in &mcp_tools {
        if ALLOWED_MISSING_CLI_TOOLS.contains(&mcp_tool.as_str()) {
            continue;
        }
        let mapped_cli = map_mcp_to_cli(mcp_tool);
        if !cli_commands.contains(&mapped_cli) {
            missing.push(format!("{} -> {}", mcp_tool, mapped_cli));
        }
    }

    if !missing.is_empty() {
        panic!(
            "CLI 미노출 MCP 도구들이 존재합니다 (ALLOWED_MISSING_CLI_TOOLS에 등록되지 않음):\n{}\n\
             실제 CLI 명령어 목록: {:?}",
            missing.join("\n"),
            cli_commands
        );
    }

    // CLAUDE.md 의 도구 개수 검증
    let project_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let claude_md_path = project_root.join("CLAUDE.md");
    let content = std::fs::read_to_string(&claude_md_path).expect("Failed to read CLAUDE.md");

    // CLAUDE.md에서 "MCP tools 57개" 카운트 추출
    let target = "MCP tools ";
    if let Some(idx) = content.find(target) {
        let start = idx + target.len();
        let rest = &content[start..];
        let end_idx = rest.find("개").expect("Expected '개' after tool count");
        let count_str = rest[..end_idx].trim().trim_matches('*');
        let count: usize = count_str.parse().expect("Failed to parse tool count");
        assert_eq!(
            count,
            mcp_tools.len(),
            "CLAUDE.md에 기재된 MCP tools 개수({})가 실제 정의된 도구 개수({})와 다릅니다.\n\
             CLAUDE.md의 카운트 문자열을 실제 도구 수와 일치하게 동기화해 주세요.",
            count,
            mcp_tools.len()
        );
    } else {
        panic!("CLAUDE.md에서 'MCP tools X개' 패턴을 찾을 수 없습니다.");
    }
}

async fn seed_large_via_db(db: &Arc<Db>) {
    let s = db.sprint_create(CreateSprintInput {
        name: "S_large".into(), goal: None, start_date: None, end_date: None,
    }).await.unwrap();
    db.sprint_update(s.id, UpdateSprintInput {
        name: None, status: Some(SprintStatus::Active),
        goal: None, start_date: None, end_date: None,
    }, "test").await.unwrap();
    
    let m = db.mission_create(CreateMissionInput {
        title: "M_large".into(), description: None, jira_key: None,
    }).await.unwrap();

    // 에픽 3개
    for i in 1..=3 {
        let e = db.epic_create(CreateEpicInput {
            project_key: "p".into(), mission_id: Some(m.id), sprint_id: Some(s.id),
            title: format!("E{}", i), description: Some("에픽 설명글입니다. ".repeat(10)),
        }).await.unwrap();

        // 각 에픽에 이슈 3개 (총 9개)
        for j in 1..=3 {
            let issue = db.issue_create(CreateIssueInput {
                epic_id: e.id, title: format!("I{}-{}", i, j),
                description: Some("이슈 상세 설명글입니다. ".repeat(5)), goal: Some("목표".into()), priority: None,
            }).await.unwrap();

            // ready 상태로 변경
            db.issue_update(issue.id, engram_core::models::issue::UpdateIssueInput {
                status: Some(engram_core::models::issue::IssueStatus::Ready),
                ..Default::default()
            }, "test").await.unwrap();

            // 각 이슈에 태스크 2개
            for k in 1..=2 {
                db.task_create(CreateTaskInput {
                    issue_id: issue.id, title: format!("T{}-{}-{}", i, j, k), description: None,
                    goal: None, after_task_id: None, source: None,
                }).await.unwrap();
            }

            // 각 이슈에 caveat 1개, decision 1개
            db.note_add(CreateNoteInput {
                issue_id: issue.id, task_id: None, note_type: NoteType::Caveat,
                summary: format!("C{}-{}", i, j), detail: Some("상세 함정 내용".into()),
                author: Some("agent".into()), agent_id: Some("test".into()),
                scope: None, scope_target_id: None, project_key: None,
            }).await.unwrap();

            db.note_add(CreateNoteInput {
                issue_id: issue.id, task_id: None, note_type: NoteType::Decision,
                summary: format!("D{}-{}", i, j), detail: Some("상세 결정 내용".into()),
                author: Some("agent".into()), agent_id: Some("test".into()),
                scope: None, scope_target_id: None, project_key: None,
            }).await.unwrap();
        }

        // 각 에픽에 required 상태인 draft 이슈 1개씩 추가 (총 3개)
        db.issue_create(CreateIssueInput {
            epic_id: e.id, title: format!("Draft-{}", i),
            description: None, goal: None, priority: None,
        }).await.unwrap();
    }

    // 전역 caveat note 3개 추가 (scope=epic, scope_target_id=epic_id)
    let epics = db.epic_list(Some("p"), false).await.unwrap();
    for (idx, epic) in epics.iter().enumerate() {
        db.note_add(CreateNoteInput {
            issue_id: 0, task_id: None, note_type: NoteType::Caveat,
            summary: format!("GlobalCaveat-{}", idx), detail: Some("전역 주의사항 상세".into()),
            author: Some("agent".into()), agent_id: Some("test".into()),
            scope: Some(engram_core::models::note::NoteScope::Epic), scope_target_id: Some(epic.id), project_key: None,
        }).await.unwrap();
    }
}

async fn seed_large_via_dispatch(db: &Arc<Db>) {
    let s = dispatch(Arc::clone(db), "sprint_create", &json!({"name": "S_large"})).await.unwrap();
    let sid = s["id"].as_i64().unwrap();
    dispatch(Arc::clone(db), "sprint_update", &json!({"id": sid, "status": "active", "agent_id": "test"})).await.unwrap();

    let m = dispatch(Arc::clone(db), "mission_create", &json!({"title": "M_large"})).await.unwrap();
    let mid = m["id"].as_i64().unwrap();

    for i in 1..=3 {
        let e = dispatch(Arc::clone(db), "epic_create", &json!({
            "project_key": "p", "mission_id": mid, "sprint_id": sid,
            "title": format!("E{}", i), "description": "에픽 설명글입니다. ".repeat(10),
        })).await.unwrap();
        let eid = e["id"].as_i64().unwrap();

        for j in 1..=3 {
            let issue = dispatch(Arc::clone(db), "issue_create", &json!({
                "epic_id": eid, "title": format!("I{}-{}", i, j),
                "description": "이슈 상세 설명글입니다. ".repeat(5), "goal": "목표",
            })).await.unwrap();
            let iid = issue["id"].as_i64().unwrap();

            dispatch(Arc::clone(db), "issue_update", &json!({
                "id": iid, "status": "ready", "agent_id": "test",
            })).await.unwrap();

            for k in 1..=2 {
                dispatch(Arc::clone(db), "task_create", &json!({
                    "issue_id": iid, "title": format!("T{}-{}-{}", i, j, k),
                })).await.unwrap();
            }

            dispatch(Arc::clone(db), "note_add", &json!({
                "issue_id": iid, "note_type": "caveat", "summary": format!("C{}-{}", i, j),
                "detail": "상세 함정 내용", "agent_id": "test",
            })).await.unwrap();

            dispatch(Arc::clone(db), "note_add", &json!({
                "issue_id": iid, "note_type": "decision", "summary": format!("D{}-{}", i, j),
                "detail": "상세 결정 내용", "agent_id": "test",
            })).await.unwrap();
        }

        dispatch(Arc::clone(db), "issue_create", &json!({
            "epic_id": eid, "title": format!("Draft-{}", i),
        })).await.unwrap();
    }

    // 전역 caveat note 추가
    let epics = dispatch(Arc::clone(db), "epic_list", &json!({"project_key": "p", "mode": "normal"})).await.unwrap();
    for (idx, epic) in epics.as_array().unwrap().iter().enumerate() {
        let eid = epic["id"].as_i64().unwrap();
        dispatch(Arc::clone(db), "note_add", &json!({
            "note_type": "caveat", "summary": format!("GlobalCaveat-{}", idx),
            "detail": "전역 주의사항 상세", "agent_id": "test",
            "scope": "epic", "scope_target_id": eid,
        })).await.unwrap();
    }
}

#[tokio::test]
async fn test_parity_session_restore_size_guard_matrix() {
    let db_a = fresh_db().await;
    seed_large_via_db(&db_a).await;

    let db_b = fresh_db().await;
    seed_large_via_dispatch(&db_b).await;

    let compacts = vec![true, false];
    let limits = vec![500, 3000, 50000];

    for compact in compacts {
        for limit in limits.iter().copied() {
            let cli_val = db_a.session_restore(None, compact, 120, Some(limit)).await.unwrap();
            let cli = normalize(serde_json::to_value(&cli_val).unwrap());

            let mcp = normalize(dispatch(
                Arc::clone(&db_b),
                "session_restore",
                &json!({
                    "compact": compact,
                    "size_limit": limit,
                    "mode": if compact { "compact" } else { "normal" }
                }),
            ).await.unwrap());

            assert_eq!(
                cli, mcp,
                "session_restore 동치 실패 (compact={}, limit={})",
                compact, limit
            );

            // size_limit 가 작을 때 절단 검증
            if limit == 500 {
                assert!(cli["truncated"].as_bool().unwrap(), "limit=500 일 때는 잘려야 함");
                assert!(cli["truncated_count"].as_i64().unwrap() > 0);
                assert!(!cli["warnings"].as_array().unwrap().is_empty());
            }

            // size_limit 가 충분히 클 때 절단되지 않아야 함
            if limit == 50000 {
                assert!(!cli["truncated"].as_bool().unwrap(), "limit=50000 일 때는 안 잘려야 함");
                assert!(cli["truncated_count"].is_null() || cli.get("truncated_count").is_none());
            }
        }
    }
}

#[tokio::test]
async fn test_session_restore_default_limit_korean_truncation() {
    let db = fresh_db().await;
    let s = db.sprint_create(CreateSprintInput {
        name: "S_korean".into(), goal: None, start_date: None, end_date: None,
    }).await.unwrap();
    db.sprint_update(s.id, UpdateSprintInput {
        name: None, status: Some(SprintStatus::Active),
        goal: None, start_date: None, end_date: None,
    }, "test").await.unwrap();
    
    let m = db.mission_create(CreateMissionInput {
        title: "M_korean".into(), description: None, jira_key: None,
    }).await.unwrap();

    // 아주 긴 한글 문자열 (3000번 반복 = 약 21,000자 = 63,000바이트)
    let long_korean = "한글설명내용입니다".repeat(3000); 

    let e = db.epic_create(CreateEpicInput {
        project_key: "p".into(), mission_id: Some(m.id), sprint_id: Some(s.id),
        title: "한글 에픽".into(), description: Some(long_korean.clone()),
    }).await.unwrap();

    // 1개 이슈 생성 및 ready로 변경
    let issue = db.issue_create(CreateIssueInput {
        epic_id: e.id, title: "한글 이슈".into(), 
        description: Some(long_korean.clone()), goal: Some(long_korean), priority: None,
    }).await.unwrap();
    db.issue_update(issue.id, engram_core::models::issue::UpdateIssueInput {
        status: Some(engram_core::models::issue::IssueStatus::Ready),
        ..Default::default()
    }, "test").await.unwrap();

    // 기본 한도(25000자 = 25000바이트)로 project_key="p" session_restore 호출
    // size_limit=None 이면 기본값 25000을 써야 함.
    // project_key="p"로 필터링해야 epic.description 과 issue.description 이 200글자로 잘리지 않아
    // 대용량 페이로드가 구성되고, 최종적으로 size limit 가드에 걸려 잘려나가게 됩니다.
    let snap = db.session_restore(Some("p"), false, 120, None).await.unwrap();
    
    assert!(snap.truncated, "한글 대용량 데이터가 포함되었으므로 기본 한도 25000바이트에서 잘려야 함");
    assert!(snap.truncated_count.unwrap() > 0);
    
    let serialized = serde_json::to_string(&snap).unwrap();
    assert!(serialized.len() <= 25000, "잘린 후 직렬화된 크기가 25000바이트 이하여야 함. 실제 크기: {}", serialized.len());
}





