use serde::{Deserialize, Serialize};

/// 태스크별 테스트 체크리스트 항목.
/// Agent 가 작업 검증 항목을 누적하고, 단일/복수 단위로 체크/언체크할 수 있다.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TaskTest {
    pub id: i64,
    pub task_id: i64,
    pub label: String,
    pub checked: bool,
    pub created_at: String,
    pub checked_at: Option<String>,
}
