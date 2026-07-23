pub mod epic;
pub mod format;
pub mod history;
pub mod issue;
pub mod mission;
pub mod note;
pub mod retrospective;
pub mod session;
pub mod sprint;
pub mod task;
pub mod task_test;

#[cfg(test)]
mod dispatch_test;

use engram_core::Db;
use serde_json::{json, Value};
use std::sync::Arc;

pub fn all_tool_definitions() -> Vec<Value> {
    let mut tools = [
        sprint::tool_definitions(),
        epic::tool_definitions(),
        mission::tool_definitions(),
        issue::tool_definitions(),
        task::tool_definitions(),
        task_test::tool_definitions(),
        note::tool_definitions(),
        session::tool_definitions(),
        history::tool_definitions(),
        retrospective::tool_definitions(),
    ]
    .concat();

    for tool in &mut tools {
        if let Some(schema) = tool.get_mut("inputSchema") {
            if let Some(properties) = schema.get_mut("properties") {
                if let Some(props_obj) = properties.as_object_mut() {
                    if !props_obj.contains_key("mode") {
                        props_obj.insert(
                            "mode".to_string(),
                            json!({
                                "type": "string",
                                "enum": ["agent", "normal", "compact", "ref"],
                                "default": "agent",
                                "description": "Output format mode. 'agent' (LLM-optimized Markdown/text), 'normal' (full JSON), 'compact' (compact JSON)."
                            })
                        );
                    }
                }
            }
        }
    }
    tools
}

pub fn get_mode(args: &Value) -> engram_core::models::OutputMode {
    if let Some(m_str) = args["mode"].as_str() {
        match m_str {
            "normal" | "full" => engram_core::models::OutputMode::Normal,
            "compact" => engram_core::models::OutputMode::Compact,
            "agent" => engram_core::models::OutputMode::Agent,
            "ref" => engram_core::models::OutputMode::Ref,
            _ => engram_core::models::OutputMode::Agent,
        }
    } else if args["compact"].as_bool().unwrap_or(false) {
        engram_core::models::OutputMode::Compact
    } else {
        engram_core::models::OutputMode::Agent
    }
}


pub async fn dispatch(
    db: Arc<Db>,
    name: &str,
    args: &Value,
) -> Result<Value, engram_core::Error> {
    match name {
        // Sprint
        "sprint_create"  => sprint::create(db, args).await,
        "sprint_list"    => sprint::list(db, args).await,
        "sprint_current" => sprint::current(db, args).await,
        "sprint_update"  => sprint::update(db, args).await,
        "sprint_delete"  => sprint::delete(db, args).await,
        // Mission
        "mission_create"     => mission::mission_create(db, args).await,
        "mission_get"        => mission::mission_get(db, args).await,
        "mission_list"       => mission::mission_list(db, args).await,
        "mission_update"     => mission::mission_update(db, args).await,
        "mission_delete"     => mission::mission_delete(db, args).await,
        "mission_get_tree"   => mission::mission_get_tree(db, args).await,
        // Epic
        "epic_create"     => epic::create(db, args).await,
        "epic_get"        => epic::get(db, args).await,
        "epic_list"       => epic::list(db, args).await,
        "epic_update"     => epic::update(db, args).await,
        "epic_delete"     => epic::delete(db, args).await,
        "epic_set_sprint" => epic::set_sprint(db, args).await,
        "epic_finish"     => epic::finish(db, args).await,
        // Issue
        "issue_create" => issue::create(db, args).await,
        "issue_get"    => issue::get(db, args).await,
        "issue_list"   => issue::list(db, args).await,
        "issue_update" => issue::update(db, args).await,
        "issue_link"   => issue::link(db, args).await,
        "issue_unlink"        => issue::unlink(db, args).await,
        "issue_delete"        => issue::delete(db, args).await,
        "issue_claim"         => issue::claim(db, args).await,
        "issue_release"       => issue::release(db, args).await,
        "issue_finish"        => issue::finish(db, args).await,
        "issue_cancel"        => issue::cancel(db, args).await,
        "issue_bulk_update"   => issue::bulk_update(db, args).await,
        "my_blocked_issues"   => issue::my_blocked_issues(db, args).await,
        "stalled_issues"      => issue::stalled(db, args).await,
        "planning_review_queue" => issue::planning_review_queue(db, args).await,
        // Task
        "task_create"       => task::create(db, args).await,
        "task_list"         => task::list(db, args).await,
        "task_update"       => task::update(db, args).await,
        "task_insert_after" => task::insert_after(db, args).await,
        "task_next"         => task::next(db, args).await,
        "task_delete"       => task::delete(db, args).await,
        // Task Tests
        "task_test_add"        => task_test::add(db, args).await,
        "task_test_add_bulk"   => task_test::add_bulk(db, args).await,
        "task_test_list"       => task_test::list(db, args).await,
        "task_test_check"      => task_test::check(db, args).await,
        "task_test_check_bulk" => task_test::check_bulk(db, args).await,
        "task_test_uncheck"    => task_test::uncheck(db, args).await,
        "task_test_remove"     => task_test::remove(db, args).await,
        // Note
        "note_add"     => note::add(db, args).await,
        "note_list"    => note::list(db, args).await,
        "note_get"     => note::get(db, args).await,
        "note_resolve" => note::resolve(db, args).await,
        "note_add_bulk" => note::add_bulk(db, args).await,
        // Session
        "session_restore" => session::restore(db, args).await,
        "session_end"     => session::end(db, args).await,
        "board_status"    => session::board_status(db, args).await,
        // History (audit log read API — ADR-0009 changed_by 의 시각화)
        "history_for"       => history::for_entity(db, args).await,
        "history_by_agent"  => history::by_agent(db, args).await,
        "history_recent"    => history::recent(db, args).await,
        // Retrospective
        "retrospective_create" => retrospective::retrospective_create(db, args).await,
        "retrospective_list"   => retrospective::retrospective_list(db, args).await,
        "retrospective_get"    => retrospective::retrospective_get(db, args).await,
        "retrospective_update" => retrospective::retrospective_update(db, args).await,
        "retrospective_delete" => retrospective::retrospective_delete(db, args).await,
        "retro_action_item_add"    => retrospective::retro_action_item_add(db, args).await,
        "retro_action_item_update" => retrospective::retro_action_item_update(db, args).await,
        "retro_action_item_convert_to_issue" => retrospective::retro_action_item_convert_to_issue(db, args).await,
        _ => Err(engram_core::Error::NotFound(format!("tool:{name}"))),
    }
}

/// Value에서 i64를 강제 형변환(coercion)해서 파싱하는 헬퍼.
/// Number 타입이면 그대로 i64로 가져오고, String 타입이면 parse를 시도합니다.
pub fn parse_i64(v: &Value) -> Option<i64> {
    if v.is_number() {
        v.as_i64()
    } else if let Some(s) = v.as_str() {
        s.parse::<i64>().ok()
    } else {
        None
    }
}

/// parse_i64를 활용하여 필수 i64 인자값을 파싱합니다.
/// 파싱이 실패하면 명확한 Validation Error를 반환합니다.
pub fn parse_required_i64(v: &Value, field_name: &str) -> Result<i64, engram_core::Error> {
    parse_i64(v).ok_or_else(|| {
        engram_core::Error::Validation(format!(
            "필드 '{field_name}'의 값이 유효한 정수가 아닙니다 (전달된 값: {v})"
        ))
    })
}

/// parse_i64를 활용하여 선택적 i64 인자값을 파싱합니다.
/// 값이 Null이거나 누락된 상태가 아니면서 파싱에 실패하면 Validation Error를 반환합니다.
pub fn parse_optional_i64(v: &Value, field_name: &str) -> Result<Option<i64>, engram_core::Error> {
    if v.is_null() {
        return Ok(None);
    }
    let val = parse_i64(v).ok_or_else(|| {
        engram_core::Error::Validation(format!(
            "필드 '{field_name}'의 값이 유효한 정수가 아닙니다 (전달된 값: {v})"
        ))
    })?;
    Ok(Some(val))
}

/// Value에서 i64 배열을 강제 형변환하여 파싱하는 헬퍼.
/// 단일 정수/문자열 단건인 경우 단일 요소 배열로도 변환을 지원하며,
/// 배열일 경우 내부 요소들을 각각 coerced 파싱합니다.
pub fn parse_i64_array(v: &Value, field_name: &str) -> Result<Vec<i64>, engram_core::Error> {
    if v.is_null() {
        return Ok(vec![]);
    }
    if let Some(arr) = v.as_array() {
        let mut result = Vec::new();
        for (i, item) in arr.iter().enumerate() {
            let val = parse_i64(item).ok_or_else(|| {
                engram_core::Error::Validation(format!(
                    "필드 '{field_name}' 배열의 {i}번째 요소가 유효한 정수가 아닙니다 (전달된 값: {item})"
                ))
            })?;
            result.push(val);
        }
        Ok(result)
    } else if let Some(n) = parse_i64(v) {
        Ok(vec![n])
    } else {
        Err(engram_core::Error::Validation(format!(
            "필드 '{field_name}'은(는) 정수 또는 정수 배열이어야 합니다 (전달된 값: {v})"
        )))
    }
}

