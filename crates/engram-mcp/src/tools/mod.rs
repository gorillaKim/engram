pub mod epic;
pub mod format;
pub mod history;
pub mod issue;
pub mod mission;
pub mod note;
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
                                "enum": ["agent", "normal", "compact"],
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
        _ => Err(engram_core::Error::NotFound(format!("tool:{name}"))),
    }
}
