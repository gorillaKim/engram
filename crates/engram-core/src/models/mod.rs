pub mod epic;
pub mod history;
pub mod issue;
pub mod mission;
pub mod note;
pub mod sprint;
pub mod task;
pub mod task_test;

pub use epic::{Epic, EpicStatus, CreateEpicInput, UpdateEpicInput};
pub use history::{History, EntityType};
pub use issue::{Issue, IssueStatus, IssuePriority, CreateIssueInput, UpdateIssueInput, IssueLink, LinkType, IssueFilter};
pub use mission::{Mission, MissionStatus, MissionSummary, CreateMissionInput, UpdateMissionInput, MissionFilter, MissionProgress, EpicWithIssues, MissionTree};
pub use note::{Note, NoteSummary, NoteType, NoteScope, CreateNoteInput};
pub use sprint::{Sprint, SprintStatus, CreateSprintInput, UpdateSprintInput};
pub use task::{Task, TaskStatus, TaskSource, CreateTaskInput, UpdateTaskInput};
pub use task_test::TaskTest;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputMode {
    Normal,
    Compact,
    Agent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CoreResponse<T> {
    Json(T),
    Text(String),
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub has_more: bool,
}

pub fn apply_projection(mut val: serde_json::Value, fields: &[String]) -> serde_json::Value {
    if fields.is_empty() {
        return val;
    }
    if let Some(obj) = val.as_object_mut() {
        if let Some(items) = obj.get_mut("items") {
            if let Some(arr) = items.as_array_mut() {
                for item in arr {
                    if let Some(item_obj) = item.as_object_mut() {
                        item_obj.retain(|k, _| fields.contains(k));
                    }
                }
            }
        } else {
            obj.retain(|k, _| fields.contains(k));
        }
    }
    val
}
