use engram_core::{Db, models::task::*};
use serde_json::{json, Value};
use std::sync::Arc;

pub fn tool_definitions() -> Vec<Value> {
    vec![
        json!({ "name": "task_create", "description": "이슈 하위에 태스크를 생성합니다.",
            "inputSchema": { "type": "object", "required": ["issue_id", "title"],
                "properties": {
                    "issue_id":      { "type": "integer" },
                    "title":         { "type": "string" },
                    "description":   { "type": "string" },
                    "after_task_id": { "type": "integer", "description": "이 태스크 다음에 삽입. 미입력 시 마지막에 추가" }
                }
            }
        }),
        json!({ "name": "task_list", "description": "이슈의 태스크 목록을 순서대로 조회합니다.",
            "inputSchema": { "type": "object", "required": ["issue_id"],
                "properties": {
                    "issue_id": { "type": "integer" },
                    "status": { "type": "string" },
                    "mode": {
                        "type": "string",
                        "enum": ["normal", "compact", "agent"],
                        "description": "출력 모드. 기본값은 'agent' (영문 요약 텍스트). 'compact' 또는 'normal' 선택 가능"
                    }
                }
            }
        }),
        json!({ "name": "task_update", "description": "태스크 상태/정보를 수정합니다. agent_id 를 명시하면 history.changed_by 로 저장됩니다.",
            "inputSchema": { "type": "object", "required": ["id", "agent_id"],
                "properties": {
                    "id":       { "type": "integer" },
                    "status":   { "type": "string" },
                    "title":    { "type": "string" },
                    "agent_id": { "type": "string" }
                }
            }
        }),
        json!({ "name": "task_delete", "description": "태스크를 삭제합니다. 연결된 task_tests 는 같이 삭제되고, notes.task_id 는 NULL 로 풀립니다.",
            "inputSchema": { "type": "object", "required": ["id"],
                "properties": { "id": { "type": "integer" } }
            }
        }),
        json!({ "name": "task_insert_after", "description": "특정 태스크 다음에 새 태스크를 삽입합니다. 작업 중 발견된 태스크에 사용하세요 (source=agent_discovered 자동 설정).",
            "inputSchema": { "type": "object", "required": ["issue_id", "after_task_id", "title"],
                "properties": {
                    "issue_id":      { "type": "integer" },
                    "after_task_id": { "type": "integer" },
                    "title":         { "type": "string" },
                    "description":   { "type": "string" }
                }
            }
        }),
        json!({ "name": "task_next",
            "description": "다음에 처리할 태스크를 우선순위 알고리즘으로 반환합니다 (블로킹 해소 → priority → in_progress 이슈 우선 → created_at). project_key로 특정 프로젝트만 필터링 가능합니다.\n\n⚠️ **이 도구는 read-only 입니다.** 반환된 task 의 부모 이슈를 working 으로 전이하려면 반드시 `issue_claim(parent_issue_id, agent_id)` 으로 atomic CAS 점유 후 진행하세요. 멀티 에이전트가 동시에 task_next 를 호출하면 같은 task 를 받을 수 있으므로 race 방지의 유일한 경로는 `issue_claim` 입니다.",
            "inputSchema": { "type": "object",
                "properties": {
                    "project_key": { "type": "string" },
                    "issue_id":    { "type": "integer", "description": "특정 이슈로 제한" },
                    "mode": {
                        "type": "string",
                        "enum": ["normal", "compact", "agent"],
                        "description": "출력 모드. 기본값은 'agent' (영문 요약 텍스트). 'compact' 또는 'normal' 선택 가능"
                    }
                }
            }
        }),
    ]
}

pub async fn create(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let input = CreateTaskInput {
        issue_id:     args["issue_id"].as_i64().unwrap_or(0),
        title:        args["title"].as_str().unwrap_or("").to_string(),
        description:  args["description"].as_str().map(String::from),
        goal:         args["goal"].as_str().map(String::from),
        after_task_id: args["after_task_id"].as_i64(),
        source:       None,
    };
    Ok(serde_json::to_value(db.task_create(input).await?).unwrap())
}

pub async fn list(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let issue_id = args["issue_id"].as_i64().unwrap_or(0);
    let mode = if let Some(m_str) = args["mode"].as_str() {
        match m_str {
            "normal" => engram_core::models::OutputMode::Normal,
            "compact" => engram_core::models::OutputMode::Compact,
            "agent" => engram_core::models::OutputMode::Agent,
            _ => engram_core::models::OutputMode::Agent,
        }
    } else {
        engram_core::models::OutputMode::Agent
    };
    let response = db.task_list_mode(issue_id, mode).await?;
    Ok(serde_json::to_value(response).unwrap())
}

pub async fn update(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let id = args["id"].as_i64().unwrap_or(0);
    let status: Option<TaskStatus> = args["status"].as_str()
        .and_then(|s| serde_json::from_value(Value::String(s.to_string())).ok());
    let agent_id = args["agent_id"].as_str()
        .ok_or_else(|| engram_core::Error::Validation("agent_id is required".to_string()))?;
    let input = UpdateTaskInput {
        title:       args["title"].as_str().map(String::from),
        description: args["description"].as_str().map(String::from),
        goal:        args["goal"].as_str().map(String::from),
        status,
    };
    Ok(serde_json::to_value(db.task_update(id, input, agent_id).await?).unwrap())
}

pub async fn insert_after(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let input = CreateTaskInput {
        issue_id:     args["issue_id"].as_i64().unwrap_or(0),
        title:        args["title"].as_str().unwrap_or("").to_string(),
        description:  args["description"].as_str().map(String::from),
        goal:         args["goal"].as_str().map(String::from),
        after_task_id: args["after_task_id"].as_i64(),
        source:       Some(TaskSource::AgentDiscovered),
    };
    Ok(serde_json::to_value(db.task_create(input).await?).unwrap())
}

pub async fn next(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let project_key = args["project_key"].as_str();
    let issue_id    = args["issue_id"].as_i64();
    let mode = if let Some(m_str) = args["mode"].as_str() {
        match m_str {
            "normal" => engram_core::models::OutputMode::Normal,
            "compact" => engram_core::models::OutputMode::Compact,
            "agent" => engram_core::models::OutputMode::Agent,
            _ => engram_core::models::OutputMode::Agent,
        }
    } else {
        engram_core::models::OutputMode::Agent
    };
    let response = db.task_next_mode(project_key, issue_id, mode).await?;
    Ok(serde_json::to_value(response).unwrap())
}

pub async fn delete(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let id = args["id"].as_i64()
        .ok_or_else(|| engram_core::Error::Validation("id is required".to_string()))?;
    db.task_delete(id).await?;
    Ok(json!({ "ok": true, "deleted_id": id }))
}
