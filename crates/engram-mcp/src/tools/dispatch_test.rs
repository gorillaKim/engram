//! MCP dispatch round-trip 통합 테스트.
//!
//! `.claude/rules/mcp-tool-shape.md` 의 요구사항: 새 MCP 도구 추가 시 dispatch
//! 직렬화 round-trip 을 검증해야 한다. 기존 도구의 회귀 방지도 함께 담는다.

#![cfg(test)]

use super::{all_tool_definitions, dispatch};
use engram_core::Db;
use serde_json::{json, Value};
use std::sync::Arc;

async fn setup() -> Arc<Db> {
    Arc::new(Db::open_in_memory().await.unwrap())
}

/// 활성 스프린트 + 에픽 + 이슈를 만들어두는 헬퍼.
async fn seed(db: &Arc<Db>) -> (i64, i64, i64) {
    let sprint = dispatch(Arc::clone(db), "sprint_create", &json!({"name": "S1"}))
        .await
        .unwrap();
    let sprint_id = sprint["id"].as_i64().unwrap();
    dispatch(
        Arc::clone(db),
        "sprint_update",
        &json!({"id": sprint_id, "status": "active"}),
    )
    .await
    .unwrap();
    let epic = dispatch(
        Arc::clone(db),
        "epic_create",
        &json!({"sprint_id": sprint_id, "project_key": "p", "title": "E"}),
    )
    .await
    .unwrap();
    let epic_id = epic["id"].as_i64().unwrap();
    let issue = dispatch(
        Arc::clone(db),
        "issue_create",
        &json!({"epic_id": epic_id, "title": "I"}),
    )
    .await
    .unwrap();
    let issue_id = issue["id"].as_i64().unwrap();
    (sprint_id, epic_id, issue_id)
}

#[tokio::test]
async fn test_all_tool_definitions_serializable_and_named() {
    let defs = all_tool_definitions();
    assert!(!defs.is_empty(), "도구 목록이 비어 있으면 안 됨");
    for d in &defs {
        let name = d["name"].as_str().expect("name 필드 필수");
        assert!(!name.is_empty(), "도구 이름은 공백일 수 없음");
        assert!(d["description"].is_string(), "description 필수: {name}");
        assert!(d["inputSchema"].is_object(), "inputSchema 필수: {name}");
    }
    let names: Vec<&str> = defs.iter().filter_map(|d| d["name"].as_str()).collect();
    for required in [
        "sprint_create",
        "sprint_update",
        "epic_create",
        "epic_update",
        "issue_create",
        "issue_update",
        "issue_link",
        "issue_unlink",
        "task_create",
        "task_update",
        "task_insert_after",
        "task_next",
        "note_add",
        "note_get",
        "note_add_bulk",
        "session_restore",
        "session_end",
        "board_status",
        "my_blocked_issues",
        "planning_review_queue",
    ] {
        assert!(names.contains(&required), "도구 누락: {required}");
    }
}

#[tokio::test]
async fn test_sprint_update_changes_status() {
    let db = setup().await;
    let sprint = dispatch(Arc::clone(&db), "sprint_create", &json!({"name": "S"}))
        .await
        .unwrap();
    let id = sprint["id"].as_i64().unwrap();
    assert_eq!(sprint["status"], "planning");

    let updated = dispatch(
        Arc::clone(&db),
        "sprint_update",
        &json!({"id": id, "status": "active"}),
    )
    .await
    .unwrap();
    assert_eq!(updated["status"], "active", "sprint_update가 status를 반영해야 함");
}

#[tokio::test]
async fn test_epic_update_changes_status() {
    let db = setup().await;
    let (_, epic_id, _) = seed(&db).await;
    let updated = dispatch(
        Arc::clone(&db),
        "epic_update",
        &json!({"id": epic_id, "status": "completed"}),
    )
    .await
    .unwrap();
    assert_eq!(updated["status"], "completed", "epic_update가 status를 반영해야 함");
}

#[tokio::test]
async fn test_task_update_changes_status() {
    let db = setup().await;
    let (_, _, issue_id) = seed(&db).await;
    let task = dispatch(
        Arc::clone(&db),
        "task_create",
        &json!({"issue_id": issue_id, "title": "T"}),
    )
    .await
    .unwrap();
    let task_id = task["id"].as_i64().unwrap();
    assert_eq!(task["status"], "required");

    let updated = dispatch(
        Arc::clone(&db),
        "task_update",
        &json!({"id": task_id, "status": "ready"}),
    )
    .await
    .unwrap();
    assert_eq!(updated["status"], "ready", "task_update가 status를 반영해야 함");
}

#[tokio::test]
async fn test_issue_link_and_unlink_roundtrip() {
    let db = setup().await;
    let (_, epic_id, a) = seed(&db).await;
    let b = dispatch(
        Arc::clone(&db),
        "issue_create",
        &json!({"epic_id": epic_id, "title": "B"}),
    )
    .await
    .unwrap()["id"]
        .as_i64()
        .unwrap();

    let link: Value = dispatch(
        Arc::clone(&db),
        "issue_link",
        &json!({"source_id": a, "target_id": b, "link_type": "blocks"}),
    )
    .await
    .unwrap();
    let link_id = link["id"].as_i64().unwrap();
    assert_eq!(link["source_id"], a);
    assert_eq!(link["link_type"], "blocks");

    let unlink: Value = dispatch(
        Arc::clone(&db),
        "issue_unlink",
        &json!({"link_id": link_id}),
    )
    .await
    .unwrap();
    assert_eq!(unlink["ok"], true);
}

#[tokio::test]
async fn test_session_restore_via_dispatch() {
    let db = setup().await;
    let (_, _, _) = seed(&db).await;
    let snap: Value = dispatch(Arc::clone(&db), "session_restore", &json!({"project_key": "p"}))
        .await
        .unwrap();
    assert!(snap["sprint_id"].as_i64().unwrap() > 0);
    assert_eq!(snap["project_key"], "p");
    assert!(snap["active_epics"].is_array());
}

#[tokio::test]
async fn test_unknown_tool_returns_not_found() {
    let db = setup().await;
    let err = dispatch(Arc::clone(&db), "no_such_tool", &json!({}))
        .await
        .unwrap_err();
    matches!(err, engram_core::Error::NotFound(_));
}

#[tokio::test]
async fn test_note_get_returns_detail() {
    let db = setup().await;
    let (_, _, issue_id) = seed(&db).await;
    let note = dispatch(
        Arc::clone(&db),
        "note_add",
        &json!({
            "issue_id": issue_id,
            "note_type": "caveat",
            "summary": "주의",
            "detail": "긴 본문"
        }),
    )
    .await
    .unwrap();
    let note_id = note["id"].as_i64().unwrap();

    let got = dispatch(Arc::clone(&db), "note_get", &json!({"note_id": note_id}))
        .await
        .unwrap();
    assert_eq!(got["summary"], "주의");
    assert_eq!(got["detail"], "긴 본문");
}

#[tokio::test]
async fn test_planning_review_queue_via_dispatch() {
    let db = setup().await;
    let (sprint_id, _, issue_id) = seed(&db).await;

    dispatch(
        Arc::clone(&db),
        "issue_set_sprint",
        &json!({
            "id": issue_id,
            "sprint_id": sprint_id
        })
    ).await.unwrap();

    dispatch(
        Arc::clone(&db),
        "issue_update",
        &json!({
            "id": issue_id,
            "status": "ready",
            "goal": "Test Goal",
            "description": "Very long description that should be excerpted. ".repeat(5)
        })
    ).await.unwrap();

    let queue: Value = dispatch(
        Arc::clone(&db),
        "planning_review_queue",
        &json!({
            "project_key": "p",
            "sprint_id": sprint_id,
            "statuses": ["ready"]
        })
    ).await.unwrap();

    assert_eq!(queue["sprint_id"].as_i64().unwrap(), sprint_id);
    let issues = queue["issues"].as_array().unwrap();
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0]["id"].as_i64().unwrap(), issue_id);
    assert_eq!(issues[0]["title"], "I");
    assert_eq!(issues[0]["status"], "ready");
    assert!(issues[0]["description_excerpt"].as_str().unwrap().contains("excerpted"));
}

#[tokio::test]
async fn test_note_add_bulk_via_dispatch() {
    let db = setup().await;
    let (_, _, issue_id) = seed(&db).await;
    let res = dispatch(
        Arc::clone(&db),
        "note_add_bulk",
        &json!({
            "notes": [
                {
                    "issue_id": issue_id,
                    "note_type": "decision",
                    "summary": "D1",
                    "detail": "Decision Detail"
                },
                {
                    "issue_id": issue_id,
                    "note_type": "caveat",
                    "summary": "C1"
                }
            ]
        })
    ).await.unwrap();

    let arr = res.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["summary"], "D1");
    assert_eq!(arr[0]["note_type"], "decision");
    assert_eq!(arr[0]["detail"], "Decision Detail");
    assert_eq!(arr[1]["summary"], "C1");
    assert_eq!(arr[1]["note_type"], "caveat");
}
