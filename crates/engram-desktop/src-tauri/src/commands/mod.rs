pub mod issue;
pub mod task;
pub mod session;
pub mod sprint;
pub mod mission;
pub mod epic;
pub mod note;
pub mod retrospective;
pub mod mcp;
pub mod settings;

pub use issue::*;
pub use task::*;
pub use session::*;
pub use sprint::*;
pub use mission::*;
pub use epic::*;
pub use note::*;
pub use retrospective::*;
pub use mcp::*;
pub use settings::*;

#[tauri::command(rename_all = "snake_case")]
pub async fn open_url(url: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(&url).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd").args(["/C", "start", "", &url]).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
    }
    Ok(())
}

pub(crate) fn parse<T: serde::de::DeserializeOwned>(s: &str) -> engram_core::Result<T> {
    serde_json::from_value(serde_json::Value::String(s.to_string()))
        .map_err(|_| engram_core::Error::Validation(format!("unknown value: {s}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use engram_core::{
        Db,
        models::{
            sprint::{CreateSprintInput, UpdateSprintInput, SprintStatus},
            epic::CreateEpicInput,
            issue::{CreateIssueInput, IssueFilter, IssueStatus, IssuePriority, UpdateIssueInput},
            task::{CreateTaskInput, TaskStatus},
            note::{CreateNoteInput, NoteType},
        },
    };

    async fn setup() -> Db {
        Db::open_in_memory().await.unwrap()
    }

    async fn seed_issue(db: &Db) -> (i64, i64) {
        let sprint = db.sprint_create(CreateSprintInput {
            name: "S1".to_string(), goal: None, start_date: None, end_date: None,
        }).await.unwrap();
        db.sprint_update(sprint.id, UpdateSprintInput {
            status: Some(SprintStatus::Active), ..Default::default()
        }, "agent").await.unwrap();
        
        let mission = db.mission_create(engram_core::models::mission::CreateMissionInput {
            title: "M1".to_string(),
            description: None,
            jira_key: None,
        }).await.unwrap();

        let epic = db.epic_create(CreateEpicInput {
            project_key: "proj".to_string(),
            mission_id: Some(mission.id),
            sprint_id: Some(sprint.id),
            title: "E1".to_string(), description: None,
        }).await.unwrap();

        let issue = db.issue_create(CreateIssueInput {
            epic_id: epic.id, title: "I1".to_string(),
            description: None, goal: None, priority: None,
        }).await.unwrap();
        (epic.id, issue.id)
    }

    #[tokio::test]
    async fn test_session_restore_command_returns_snapshot() {
        let db = setup().await;
        assert!(do_session_restore(&db, None).await.is_ok());
    }

    #[tokio::test]
    async fn test_board_status_command_returns_board() {
        let db = setup().await;
        assert!(do_board_status(&db, None).await.is_ok());
    }

    #[tokio::test]
    async fn test_issue_list_command_returns_vec() {
        let db = setup().await;
        let (epic_id, _) = seed_issue(&db).await;
        let issues = do_issue_list(&db, IssueFilter { epic_id: Some(epic_id), ..Default::default() }).await.unwrap();
        assert_eq!(issues.len(), 1);
    }

    #[tokio::test]
    async fn test_sprint_current_returns_active_sprint() {
        let db = setup().await;
        assert!(do_sprint_current(&db).await.unwrap().is_none());
        let sprint = db.sprint_create(CreateSprintInput {
            name: "Active".to_string(), goal: None, start_date: None, end_date: None,
        }).await.unwrap();
        db.sprint_update(sprint.id, UpdateSprintInput {
            status: Some(SprintStatus::Active), ..Default::default()
        }, "agent").await.unwrap();
        assert_eq!(do_sprint_current(&db).await.unwrap().unwrap().id, sprint.id);
    }

    #[tokio::test]
    async fn test_issue_set_status_valid_transition() {
        let db = setup().await;
        let (_, issue_id) = seed_issue(&db).await;
        let updated = do_issue_set_status(&db, issue_id, "ready").await.unwrap();
        assert_eq!(updated.status, IssueStatus::Ready);
    }

    #[tokio::test]
    async fn test_issue_set_status_any_transition_allowed() {
        let db = setup().await;
        let (_, issue_id) = seed_issue(&db).await;
        let updated = do_issue_set_status(&db, issue_id, "working").await.unwrap();
        assert_eq!(updated.status, IssueStatus::Working);

        let reverted = do_issue_set_status(&db, issue_id, "required").await.unwrap();
        assert_eq!(reverted.status, IssueStatus::Required);
    }

    #[tokio::test]
    async fn test_issue_set_status_unknown_value_returns_err() {
        let db = setup().await;
        let (_, issue_id) = seed_issue(&db).await;
        assert!(do_issue_set_status(&db, issue_id, "bogus").await.is_err());
    }

    #[tokio::test]
    async fn test_task_list_and_set_status() {
        let db = setup().await;
        let (_, issue_id) = seed_issue(&db).await;
        db.task_create(CreateTaskInput {
            issue_id, title: "T1".to_string(),
            description: None, goal: None, after_task_id: None, source: None,
        }).await.unwrap();
        let tasks = do_task_list(&db, issue_id).await.unwrap();
        assert_eq!(tasks.len(), 1);

        let updated = do_task_set_status(&db, tasks[0].id, "finished").await.unwrap();
        assert_eq!(updated.status, TaskStatus::Finished);
    }

    #[tokio::test]
    async fn test_note_add_list_get_resolve() {
        let db = setup().await;
        let (_, issue_id) = seed_issue(&db).await;

        let note = do_note_add(&db, CreateNoteInput {
            issue_id, task_id: None,
            note_type: NoteType::Context,
            summary: "context note".to_string(),
            detail: Some("detail text".to_string()),
            author: None,
            agent_id: None,
            scope: None, scope_target_id: None, project_key: None,
        }).await.unwrap();
        assert_eq!(note.summary, "context note");

        let notes = do_note_list(&db, Some(issue_id), None, None).await.unwrap();
        assert_eq!(notes.len(), 1);

        let fetched = do_note_get(&db, note.id).await.unwrap();
        assert_eq!(fetched.detail, Some("detail text".to_string()));

        let resolved = do_note_resolve(&db, note.id).await.unwrap();
        assert!(resolved.resolved);
    }

    #[tokio::test]
    async fn test_epic_and_mission_note_add_and_list() {
        use engram_core::models::note::NoteScope;
        let db = setup().await;
        let mission = db.mission_create(engram_core::models::mission::CreateMissionInput {
            title: "Test Mission".to_string(),
            description: None,
            jira_key: None,
        }).await.unwrap();

        let epic = db.epic_create(engram_core::models::epic::CreateEpicInput {
            project_key: "proj".to_string(),
            mission_id: Some(mission.id),
            sprint_id: None,
            title: "Test Epic".to_string(),
            description: None,
        }).await.unwrap();

        // Epic Note Add
        let epic_note = do_note_add(&db, CreateNoteInput {
            issue_id: 0,
            task_id: None,
            note_type: NoteType::Caveat,
            summary: "Epic Note".to_string(),
            detail: Some("Epic note detail".to_string()),
            author: Some("user".to_string()),
            agent_id: None,
            scope: Some(NoteScope::Epic),
            scope_target_id: Some(epic.id),
            project_key: None,
        }).await.unwrap();
        assert_eq!(epic_note.summary, "Epic Note");

        let epic_notes = do_note_list(&db, None, Some(epic.id), None).await.unwrap();
        assert_eq!(epic_notes.len(), 1);
        assert_eq!(epic_notes[0].id, epic_note.id);

        // Mission Note Add
        let mission_note = do_note_add(&db, CreateNoteInput {
            issue_id: 0,
            task_id: None,
            note_type: NoteType::Decision,
            summary: "Mission Note".to_string(),
            detail: Some("Mission note detail".to_string()),
            author: Some("user".to_string()),
            agent_id: None,
            scope: Some(NoteScope::Mission),
            scope_target_id: Some(mission.id),
            project_key: None,
        }).await.unwrap();
        assert_eq!(mission_note.summary, "Mission Note");

        let mission_notes = do_note_list(&db, None, None, Some(mission.id)).await.unwrap();
        assert_eq!(mission_notes.len(), 1);
        assert_eq!(mission_notes[0].id, mission_note.id);
    }

    #[tokio::test]
    async fn test_blocked_issues_graph_empty() {
        let db = setup().await;
        let graph = do_blocked_issues_graph(&db, "proj").await.unwrap();
        assert!(graph.chains.is_empty());
        assert!(!graph.has_cycle);
    }

    #[tokio::test]
    async fn test_issue_update_title_description_goal() {
        let db = setup().await;
        let (_, issue_id) = seed_issue(&db).await;

        let updated = do_issue_update(
            &db, issue_id,
            Some("Updated Title".to_string()),
            Some("Updated desc".to_string()),
            Some("New goal".to_string()),
            None,
        ).await.unwrap();

        assert_eq!(updated.title, "Updated Title");
        assert_eq!(updated.description, Some("Updated desc".to_string()));
        assert_eq!(updated.goal, Some("New goal".to_string()));
    }

    #[tokio::test]
    async fn test_issue_update_partial_only_description() {
        let db = setup().await;
        let (_, issue_id) = seed_issue(&db).await;

        let updated = do_issue_update(&db, issue_id, None, Some("desc only".to_string()), None, None).await.unwrap();
        assert_eq!(updated.title, "I1", "title unchanged");
        assert_eq!(updated.description, Some("desc only".to_string()));
        assert_eq!(updated.goal, None, "goal unchanged");
    }

    #[tokio::test]
    async fn test_issue_update_clear_description_with_empty_string() {
        let db = setup().await;
        let (_, issue_id) = seed_issue(&db).await;

        do_issue_update(&db, issue_id, None, Some("initial desc".to_string()), None, None).await.unwrap();
        let cleared = do_issue_update(&db, issue_id, None, Some("".to_string()), None, None).await.unwrap();
        assert!(cleared.description.as_deref().unwrap_or("").is_empty());
    }

    #[test]
    fn test_tauri_issue_filter_deserialization_parity() {
        let json_data = r#"{
            "epic_id": 123,
            "mission_id": 456,
            "sprint_id": 789,
            "backlog_only": true,
            "project_key": "proj",
            "status": "ready",
            "statuses": ["working", "demo"],
            "priority": "high",
            "compact": true,
            "limit": 10,
            "offset": 0,
            "updated_after": "2026-06-23T00:00:00Z"
        }"#;

        let filter: IssueFilter = serde_json::from_str(json_data).unwrap();
        assert_eq!(filter.epic_id, Some(123));
        assert_eq!(filter.mission_id, Some(456));
        assert_eq!(filter.sprint_id, Some(789));
        assert!(filter.backlog_only);
        assert_eq!(filter.project_key.as_deref(), Some("proj"));
        assert_eq!(filter.status, Some(IssueStatus::Ready));
        assert_eq!(filter.statuses, Some(vec![IssueStatus::Working, IssueStatus::Demo]));
        assert_eq!(filter.priority, Some(IssuePriority::High));
        assert_eq!(filter.compact, Some(true));
        assert_eq!(filter.limit, Some(10));
        assert_eq!(filter.offset, Some(0));
        assert_eq!(filter.updated_after.as_deref(), Some("2026-06-23T00:00:00Z"));
    }

    #[test]
    fn test_tauri_update_issue_input_deserialization_parity() {
        let json_data = r#"{
            "title": "New Title",
            "description": "New Description",
            "goal": "New Goal",
            "status": "working",
            "priority": "critical",
            "epic_id": 999
        }"#;

        let input: UpdateIssueInput = serde_json::from_str(json_data).unwrap();
        assert_eq!(input.title.as_deref(), Some("New Title"));
        assert_eq!(input.description.as_deref(), Some("New Description"));
        assert_eq!(input.goal.as_deref(), Some("New Goal"));
        assert_eq!(input.status, Some(IssueStatus::Working));
        assert_eq!(input.priority, Some(IssuePriority::Critical));
        assert_eq!(input.epic_id, Some(999));
    }
}
