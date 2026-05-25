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
        &json!({"id": sprint_id, "status": "active", "agent_id": "test"}),
    )
    .await
    .unwrap();
    let mission = dispatch(
        Arc::clone(db),
        "mission_create",
        &json!({"title": "M"}),
    )
    .await
    .unwrap();
    let mission_id = mission["id"].as_i64().unwrap();
    let epic = dispatch(
        Arc::clone(db),
        "epic_create",
        &json!({"project_key": "p", "title": "E", "mission_id": mission_id, "sprint_id": sprint_id}),
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
        &json!({"id": id, "status": "active", "agent_id": "test"}),
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
        &json!({"id": epic_id, "status": "completed", "agent_id": "test"}),
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
        &json!({"id": task_id, "status": "ready", "agent_id": "test"}),
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
        &json!({"source_id": a, "target_id": b, "link_type": "blocks", "agent_id": "test"}),
    )
    .await
    .unwrap();
    let link_id = link["id"].as_i64().unwrap();
    assert_eq!(link["source_id"], a);
    assert_eq!(link["link_type"], "blocks");

    let unlink: Value = dispatch(
        Arc::clone(&db),
        "issue_unlink",
        &json!({"link_id": link_id, "agent_id": "test"}),
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
            "detail": "긴 본문",
            "agent_id": "test"
        }),
    )
    .await
    .unwrap();
    let note_id = note["id"].as_i64().unwrap();

    let got = dispatch(Arc::clone(&db), "note_get", &json!({"id": note_id}))
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
        "issue_update",
        &json!({
            "id": issue_id,
            "status": "ready",
            "goal": "Test Goal",
            "description": "Very long description that should be excerpted. ".repeat(5),
            "agent_id": "test"
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
            "agent_id": "test",
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

// ── Issue #178: compact mode + issue_unlink delete response shape ─────────────

/// 헬퍼: issue 를 ready 상태로 승격 (seed() 는 required 로 만드므로 active_issues 에 안 보임)
async fn promote_to_ready(db: &Arc<Db>, issue_id: i64, _sprint_id: i64) {
    dispatch(
        Arc::clone(db),
        "issue_update",
        &json!({"id": issue_id, "status": "ready", "agent_id": "test"}),
    )
    .await
    .unwrap();
}

/// Test A: compact=true 응답이 full 응답보다 30% 이상 작아야 한다.
#[tokio::test]
async fn test_session_restore_compact_reduces_payload() {
    let db = setup().await;
    let (sprint_id, epic_id, issue_id) = seed(&db).await;
    promote_to_ready(&db, issue_id, sprint_id).await;

    // 3개 추가 이슈 생성 + ready 상태로 승격
    for _ in 0..2 {
        let extra = dispatch(
            Arc::clone(&db),
            "issue_create",
            &json!({"epic_id": epic_id, "title": "Extra"}),
        )
        .await
        .unwrap()["id"]
            .as_i64()
            .unwrap();
        promote_to_ready(&db, extra, sprint_id).await;
        // 각 이슈에 tasks 3개, notes 3개
        for t in 0..3 {
            dispatch(
                Arc::clone(&db),
                "task_create",
                &json!({"issue_id": extra, "title": format!("T{t}")}),
            )
            .await
            .unwrap();
        }
        for n in 0..3 {
            dispatch(
                Arc::clone(&db),
                "note_add",
                &json!({
                    "issue_id": extra,
                    "note_type": "caveat",
                    "summary": format!("note {n}"),
                    "detail": "some longer detail text to inflate the payload size considerably",
                    "agent_id": "test"
                }),
            )
            .await
            .unwrap();
        }
    }

    // 첫 번째 이슈에도 tasks/notes 추가
    for t in 0..3 {
        dispatch(
            Arc::clone(&db),
            "task_create",
            &json!({"issue_id": issue_id, "title": format!("T{t}")}),
        )
        .await
        .unwrap();
    }
    for n in 0..3 {
        dispatch(
            Arc::clone(&db),
            "note_add",
            &json!({
                "issue_id": issue_id,
                "note_type": "decision",
                "summary": format!("note {n}"),
                "detail": "some longer detail text to inflate the payload size considerably",
                "agent_id": "test"
            }),
        )
        .await
        .unwrap();
    }

    let full_resp = dispatch(Arc::clone(&db), "session_restore", &json!({"project_key": "p", "compact": false}))
        .await
        .unwrap();
    let compact_resp = dispatch(Arc::clone(&db), "session_restore", &json!({"project_key": "p", "compact": true}))
        .await
        .unwrap();

    let full_len = serde_json::to_string(&full_resp).unwrap().len();
    let compact_len = serde_json::to_string(&compact_resp).unwrap().len();

    assert!(
        compact_len < full_len,
        "compact 응답이 full 응답보다 작아야 함: compact={compact_len} full={full_len}"
    );
    assert!(
        (compact_len as f64) < (full_len as f64) * 0.7,
        "compact 는 full 의 70% 미만이어야 함: compact={compact_len} full={full_len}"
    );
}

/// Test B: compact 파라미터 미입력 시 compact=false 와 동일한 응답을 반환한다.
#[tokio::test]
async fn test_session_restore_compact_default_is_full() {
    let db = setup().await;
    let (sprint_id, _, issue_id) = seed(&db).await;
    promote_to_ready(&db, issue_id, sprint_id).await;

    let default_resp = dispatch(Arc::clone(&db), "session_restore", &json!({"project_key": "p"}))
        .await
        .unwrap();
    let explicit_full = dispatch(
        Arc::clone(&db),
        "session_restore",
        &json!({"project_key": "p", "compact": false}),
    )
    .await
    .unwrap();

    // active_issues_compact 필드가 없어야 한다 (skip_serializing_if = None)
    assert!(
        default_resp["active_epics"][0]["active_issues_compact"].is_null(),
        "기본 모드에서 active_issues_compact 가 직렬화되면 안 됨"
    );
    assert_eq!(
        default_resp["active_epics"][0]["active_issues"],
        explicit_full["active_epics"][0]["active_issues"],
        "기본 응답과 compact=false 응답의 active_issues 가 동일해야 함"
    );
}

/// Test C: compact=true 시 task_count/note_count 가 실제 full 모드의 배열 길이와 일치한다.
#[tokio::test]
async fn test_session_restore_compact_counts_accurate() {
    let db = setup().await;
    let (sprint_id, _, issue_id) = seed(&db).await;
    promote_to_ready(&db, issue_id, sprint_id).await;

    // 정확히 2개 task, 3개 note 추가
    for t in 0..2 {
        dispatch(
            Arc::clone(&db),
            "task_create",
            &json!({"issue_id": issue_id, "title": format!("Task {t}")}),
        )
        .await
        .unwrap();
    }
    for n in 0..3 {
        dispatch(
            Arc::clone(&db),
            "note_add",
            &json!({"issue_id": issue_id, "note_type": "caveat", "summary": format!("Note {n}"), "agent_id": "test"}),
        )
        .await
        .unwrap();
    }

    let compact_resp = dispatch(
        Arc::clone(&db),
        "session_restore",
        &json!({"project_key": "p", "compact": true}),
    )
    .await
    .unwrap();
    let full_resp = dispatch(
        Arc::clone(&db),
        "session_restore",
        &json!({"project_key": "p", "compact": false}),
    )
    .await
    .unwrap();

    let compact_issue = &compact_resp["active_epics"][0]["active_issues_compact"][0];
    assert_eq!(
        compact_issue["task_count"].as_i64().unwrap(),
        2,
        "compact task_count 가 2 이어야 함"
    );
    assert_eq!(
        compact_issue["note_count"].as_i64().unwrap(),
        3,
        "compact note_count 가 3 이어야 함"
    );

    let full_issue = &full_resp["active_epics"][0]["active_issues"][0];
    // full 모드의 current_task 는 ready 태스크 1개, active_notes 는 3개
    let full_notes_len = full_issue["active_notes"].as_array().unwrap().len();
    assert_eq!(
        full_notes_len, 3,
        "full 모드 active_notes 배열 길이가 3 이어야 함"
    );
}

/// Test D: issue_unlink 가 { ok: true, deleted_id: <i64> } 를 반환한다.
#[tokio::test]
async fn test_issue_unlink_returns_deleted_id() {
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

    let link = dispatch(
        Arc::clone(&db),
        "issue_link",
        &json!({"source_id": a, "target_id": b, "link_type": "blocks", "agent_id": "test"}),
    )
    .await
    .unwrap();
    let link_id = link["id"].as_i64().unwrap();

    let unlink: Value = dispatch(
        Arc::clone(&db),
        "issue_unlink",
        &json!({"link_id": link_id, "agent_id": "test"}),
    )
    .await
    .unwrap();

    assert_eq!(unlink["ok"], true, "ok 필드가 true 이어야 함");
    assert!(
        unlink["deleted_id"].is_i64(),
        "deleted_id 가 i64 이어야 함, got: {:?}",
        unlink["deleted_id"]
    );
    assert_eq!(
        unlink["deleted_id"].as_i64().unwrap(),
        link_id,
        "deleted_id 가 삭제된 link_id 와 일치해야 함"
    );
}

/// Test E: 태스크/노트가 없는 이슈의 compact 카운트는 0 이어야 한다.
#[tokio::test]
async fn test_session_restore_compact_zero_children() {
    let db = setup().await;
    let (sprint_id, _, issue_id) = seed(&db).await;
    promote_to_ready(&db, issue_id, sprint_id).await;

    let compact_resp = dispatch(
        Arc::clone(&db),
        "session_restore",
        &json!({"project_key": "p", "compact": true}),
    )
    .await
    .unwrap();

    let compact_issue = &compact_resp["active_epics"][0]["active_issues_compact"][0];
    assert_eq!(
        compact_issue["task_count"].as_i64().unwrap(),
        0,
        "태스크 없을 때 task_count=0 이어야 함"
    );
    assert_eq!(
        compact_issue["note_count"].as_i64().unwrap(),
        0,
        "노트 없을 때 note_count=0 이어야 함"
    );
    let blocked_by = compact_issue["blocked_by_ids"].as_array().unwrap();
    assert!(blocked_by.is_empty(), "블로커 없을 때 blocked_by_ids=[] 이어야 함");
}

// ── Issue #175: dispatch 무결성 ────────────────────────────────────────────────

/// Issue #175: dispatch 무결성 — tool_definitions() 에 있는 모든 도구명이
/// dispatch 에서 NotFound("tool:...") 이외의 에러(Unknown tool)를 내지 않는지 확인한다.
/// 인자 없이 호출하면 Validation 또는 NotFound(entity) 에러가 나와도 괜찮다.
/// "Unknown tool" = NotFound("tool:<name>") 형태는 절대 나오면 안 된다.
#[tokio::test]
async fn test_all_defined_tools_are_dispatchable() {
    let db = setup().await;
    let (sprint_id, epic_id, issue_id) = seed(&db).await;

    // 최소한의 더미 인자 맵 — 도구별로 필수 인자가 달라 완벽한 호출은 어렵지만,
    // "Unknown tool" 분기(NotFound("tool:..."))는 빈 args 로도 즉시 검출된다.
    let defs = all_tool_definitions();
    for def in &defs {
        let name = def["name"].as_str().unwrap();
        // 대표 인자 힌트: id/issue_id/epic_id/sprint_id 를 넣어 NotFound(entity) 가 아닌
        // "Unknown tool" 분기로 떨어지는 케이스를 걸러낸다.
        let args = json!({
            "id": issue_id,
            "issue_id": issue_id,
            "epic_id": epic_id,
            "sprint_id": sprint_id,
            "project_key": "p",
            "agent_id": "test"
        });
        let result = dispatch(Arc::clone(&db), name, &args).await;
        match result {
            Ok(_) => {} // 성공 — 당연히 OK
            Err(engram_core::Error::NotFound(ref msg)) if msg.starts_with("tool:") => {
                panic!("도구 '{name}' 이 dispatch 에서 'Unknown tool' 로 처리됨 — mod.rs 분기 누락: {msg}");
            }
            Err(_) => {} // Validation / NotFound(entity) / Conflict 등 — 인자 부족으로 인한 정상 에러
        }
    }
}
