use engram_core::{Db, models::epic::*};
use serde_json::{json, Value};
use std::sync::Arc;

pub fn tool_definitions() -> Vec<Value> {
    vec![
        json!({ "name": "epic_create",
            "description": "새 에픽을 생성합니다. mission_id 로 상위 미션을, sprint_id 로 실행 스프린트를 지정합니다 (ADR-0014: sprint SSOT 는 에픽).",
            "inputSchema": { "type": "object", "required": ["project_key", "title", "mission_id"],
                "properties": {
                    "project_key":  { "type": "string",  "description": "프로젝트 식별자 (예: 'xpert-da-web')" },
                    "mission_id":   { "type": "integer", "description": "소속 미션 ID" },
                    "sprint_id":    { "type": "integer", "description": "실행 스프린트 (생략 시 백로그)" },
                    "title":        { "type": "string" },
                    "description":  { "type": "string" }
                }
            }
        }),
        json!({ "name": "epic_get", "description": "에픽 상세를 조회합니다.",
            "inputSchema": { "type": "object", "required": ["id"],
                "properties": {
                    "id": { "type": "integer" },
                    "mode": {
                        "type": "string",
                        "enum": ["normal", "compact", "agent"],
                        "description": "출력 모드. 기본값은 'agent' (영문 요약 텍스트). 'compact' 또는 'normal' 선택 가능"
                    }
                }
            }
        }),
        json!({ "name": "epic_list",
            "description": "에픽 목록을 조회합니다. project_key / sprint_id / backlog_only / include_completed 로 필터합니다.",
            "inputSchema": { "type": "object",
                "properties": {
                    "project_key":       { "type": "string" },
                    "sprint_id":         { "type": "integer", "description": "특정 스프린트의 에픽만" },
                    "backlog_only":      { "type": "boolean", "description": "sprint_id IS NULL 만. 기본 false" },
                    "include_completed": { "type": "boolean", "description": "기본 false. true 시 completed 에픽도 포함" },
                    "detail":            { "type": "boolean", "description": "기본 false. true 시 description 본문 전체 반환" },
                    "mode": {
                        "type": "string",
                        "enum": ["normal", "compact", "agent"],
                        "description": "출력 모드. 기본값은 'agent' (영문 요약 텍스트). 'compact' 또는 'normal' 선택 가능"
                    }
                }
            }
        }),
        json!({ "name": "epic_update",
            "description": "에픽 정보(제목/설명/상태/미션/스프린트) 를 수정합니다. sprint_id 변경 시 update_sprint_id=true 를 함께 보내야 적용됩니다 (None 으로 명시 백로그 이동 가능).",
            "inputSchema": { "type": "object", "required": ["id", "agent_id"],
                "properties": {
                    "id":               { "type": "integer" },
                    "title":            { "type": "string" },
                    "description":      { "type": "string" },
                    "status":           { "type": "string" },
                    "mission_id":       { "type": "integer", "description": "미션 변경" },
                    "sprint_id":        { "type": "integer", "description": "스프린트 변경 — update_sprint_id=true 일 때만 적용. null 보내면 백로그" },
                    "update_sprint_id": { "type": "boolean", "description": "sprint_id 필드를 적용할지 여부. 기본 false" },
                    "agent_id":         { "type": "string" }
                }
            }
        }),
        json!({ "name": "epic_set_sprint",
            "description": "에픽의 소속 스프린트를 변경합니다. sprint_id 생략 시 백로그(NULL) 로 이동. 산하 모든 이슈가 자동으로 따라 옵니다 (Issue 는 epic.sprint_id 를 derive).",
            "inputSchema": { "type": "object", "required": ["epic_id", "agent_id"],
                "properties": {
                    "epic_id":   { "type": "integer" },
                    "sprint_id": { "type": "integer", "description": "생략하거나 null 이면 백로그" },
                    "agent_id":  { "type": "string" }
                }
            }
        }),
        json!({ "name": "epic_delete", "description": "에픽을 삭제합니다. 하위 이슈/태스크/노트/링크가 함께 cascade 삭제됩니다 — 비가역.",
            "inputSchema": { "type": "object", "required": ["id"],
                "properties": {
                    "id":       { "type": "integer" },
                    "agent_id": { "type": "string", "description": "호출 액터 식별자. 생략 시 'agent'." }
                }
            }
        }),
    ]
}

pub async fn create(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let mission_id = args["mission_id"].as_i64()
        .ok_or_else(|| engram_core::Error::Validation("mission_id is required".to_string()))?;
    let input = CreateEpicInput {
        project_key: args["project_key"].as_str().unwrap_or("").to_string(),
        mission_id:  Some(mission_id),
        sprint_id:   args["sprint_id"].as_i64(),
        title:       args["title"].as_str().unwrap_or("").to_string(),
        description: args["description"].as_str().map(String::from),
    };
    let epic = db.epic_create(input).await?;
    let mode = super::get_mode(args);
    if matches!(mode, engram_core::models::OutputMode::Agent) {
        Ok(super::format::success(&format!(
            "Epic #{} created.",
            epic.id
        )))
    } else {
        Ok(json!({ "id": epic.id, "status": "ok" }))
    }
}

pub async fn delete(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let id = args["id"].as_i64()
        .ok_or_else(|| engram_core::Error::Validation("id is required".to_string()))?;
    let agent_id = args["agent_id"].as_str().unwrap_or("agent");
    db.epic_delete(id, agent_id).await?;
    let mode = super::get_mode(args);
    if matches!(mode, engram_core::models::OutputMode::Agent) {
        Ok(super::format::success(&format!(
            "Epic #{} deleted.",
            id
        )))
    } else {
        Ok(json!({ "status": "ok", "deleted_id": id }))
    }
}

pub async fn get(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let id = args["id"].as_i64().unwrap_or(0);
    let mode = super::get_mode(args);
    let response = db.epic_get_mode(id, mode).await?;
    match response {
        engram_core::models::CoreResponse::Text(s) => Ok(Value::String(s)),
        engram_core::models::CoreResponse::Json(j) => Ok(serde_json::to_value(j).unwrap()),
    }
}

pub async fn list(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let project_key = args["project_key"].as_str();
    let include_completed = args["include_completed"].as_bool().unwrap_or(false);
    let mut mode = super::get_mode(args);
    if args["detail"].as_bool().unwrap_or(false) {
        mode = engram_core::models::OutputMode::Normal;
    }
    let response = db.epic_list_mode(project_key, include_completed, mode).await?;
    match response {
        engram_core::models::CoreResponse::Text(s) => Ok(Value::String(s)),
        engram_core::models::CoreResponse::Json(j) => Ok(serde_json::to_value(j).unwrap()),
    }
}

pub async fn update(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let id = args["id"].as_i64().unwrap_or(0);
    let status: Option<EpicStatus> = args["status"].as_str()
        .and_then(|s| serde_json::from_value(Value::String(s.to_string())).ok());
    let agent_id = args["agent_id"].as_str()
        .ok_or_else(|| engram_core::Error::Validation("agent_id is required".to_string()))?;
    let input = UpdateEpicInput {
        title:            args["title"].as_str().map(String::from),
        description:      args["description"].as_str().map(String::from),
        status,
        mission_id:       args["mission_id"].as_i64(),
        sprint_id:        args["sprint_id"].as_i64(),
        update_sprint_id: args["update_sprint_id"].as_bool().unwrap_or(false),
    };
    let epic = db.epic_update(id, input, agent_id).await?;
    let mode = super::get_mode(args);
    if matches!(mode, engram_core::models::OutputMode::Agent) {
        Ok(super::format::success(&format!(
            "Epic #{} updated.",
            epic.id
        )))
    } else {
        Ok(json!({ "id": epic.id, "status": "ok" }))
    }
}

pub async fn set_sprint(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let epic_id = args["epic_id"].as_i64()
        .ok_or_else(|| engram_core::Error::Validation("epic_id is required".to_string()))?;
    let sprint_id = args["sprint_id"].as_i64(); // None = 백로그
    let agent_id = args["agent_id"].as_str()
        .ok_or_else(|| engram_core::Error::Validation("agent_id is required".to_string()))?;
    let epic = db.epic_set_sprint(epic_id, sprint_id, agent_id).await?;
    let mode = super::get_mode(args);
    if matches!(mode, engram_core::models::OutputMode::Agent) {
        Ok(super::format::success(&format!(
            "Epic #{} sprint set.",
            epic.id
        )))
    } else {
        Ok(json!({ "id": epic.id, "sprint_id": epic.sprint_id, "status": "ok" }))
    }
}
