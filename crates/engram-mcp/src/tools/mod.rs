pub mod epic;
pub mod issue;
pub mod note;
pub mod session;
pub mod sprint;
pub mod task;
pub mod task_test;

#[cfg(test)]
mod dispatch_test;

use engram_core::Db;
use serde_json::Value;
use std::sync::Arc;

pub fn all_tool_definitions() -> Vec<Value> {
    [
        sprint::tool_definitions(),
        epic::tool_definitions(),
        issue::tool_definitions(),
        task::tool_definitions(),
        task_test::tool_definitions(),
        note::tool_definitions(),
        session::tool_definitions(),
    ]
    .concat()
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
        // Epic
        "epic_create"  => epic::create(db, args).await,
        "epic_get"     => epic::get(db, args).await,
        "epic_list"    => epic::list(db, args).await,
        "epic_update"       => epic::update(db, args).await,
        "epic_delete"       => epic::delete(db, args).await,
        // 백로그 / 스프린트 이동 헬퍼 (이슈가 sprint_id 를 직접 보유하는 신모델에서는
        //   epic 단위 헬퍼는 "에픽 안의 모든 이슈를 옮기는" 편의 shim 으로 동작한다)
        "epic_list_backlog" => epic::list_backlog(db, args).await,
        "epic_set_sprint"   => epic::set_sprint(db, args).await,
        // Issue
        "issue_create" => issue::create(db, args).await,
        "issue_get"    => issue::get(db, args).await,
        "issue_list"   => issue::list(db, args).await,
        "issue_update" => issue::update(db, args).await,
        "issue_link"   => issue::link(db, args).await,
        "issue_unlink"        => issue::unlink(db, args).await,
        "my_blocked_issues"   => issue::my_blocked_issues(db, args).await,
        "issue_set_sprint"    => issue::set_sprint(db, args).await,
        "stalled_issues"      => issue::stalled(db, args).await,
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
        // Session
        "session_restore" => session::restore(db, args).await,
        "session_end"     => session::end(db, args).await,
        "board_status"    => session::board_status(db, args).await,
        _ => Err(engram_core::Error::NotFound(format!("tool:{name}"))),
    }
}
