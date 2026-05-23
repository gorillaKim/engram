use serde::{Deserialize, Serialize};

use super::epic::Epic;
use super::issue::Issue;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Mission {
    pub id: i64,
    pub jira_key: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub status: MissionStatus,
    pub sprint_id: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MissionStatus {
    Active,
    Completed,
    Cancelled,
    // 자동 전이 없음 — mission_update 명시 호출로만 전환
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMissionInput {
    pub title: String,
    pub description: Option<String>,
    pub jira_key: Option<String>,
    pub sprint_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateMissionInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub jira_key: Option<String>,
    pub status: Option<MissionStatus>,
    pub sprint_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MissionFilter {
    pub sprint_id: Option<i64>,
    pub status: Option<MissionStatus>,
    /// false(기본): active만 반환. true: completed/cancelled도 포함
    #[serde(default)]
    pub include_completed: bool,
}

/// epic 하나와 그에 속한 이슈 목록.
/// `mission_get_tree` 의 epics 배열 원소.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpicWithIssues {
    pub epic: Epic,
    pub issues: Vec<Issue>,
}

/// `mission_get_tree` 의 최상위 반환 타입.
/// Mission → Vec<EpicWithIssues> 계층 트리.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionTree {
    pub mission: Mission,
    pub epics: Vec<EpicWithIssues>,
    /// missions.sprint_id 로 조회한 sprint.title. sprint_id 없으면 None.
    pub sprint_name: Option<String>,
}

/// session_restore 응답용 미션 경량 요약.
/// active 미션 목록과 진척률을 에이전트에게 제공한다.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MissionSummary {
    pub id: i64,
    pub title: String,
    pub status: MissionStatus,
    pub progress_rate: f64,
    pub epic_count: i64,
}

/// 미션별 종합 진척도 집계 결과.
/// `mission_progress_query(id)` 가 반환한다.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MissionProgress {
    pub id: i64,
    pub title: String,
    /// mission_id를 가진 에픽 수
    pub epics_count: i64,
    /// 전체 이슈 수 (cancelled 포함)
    pub issues_count: i64,
    /// status IN ('required', 'ready')
    pub todo_issues: i64,
    pub working_issues: i64,
    pub demo_issues: i64,
    pub finished_issues: i64,
    pub cancelled_issues: i64,
    /// finished / total. 분모가 0이면 0.0
    pub progress_rate: f64,
}
