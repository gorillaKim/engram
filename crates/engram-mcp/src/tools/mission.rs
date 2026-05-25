use std::sync::Arc;
use serde_json::{json, Value};
use engram_core::{Db, Error, Result};
use engram_core::models::mission::{CreateMissionInput, UpdateMissionInput, MissionFilter, MissionStatus};

pub fn tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "mission_create",
            "description": "새 미션을 생성합니다. 미션은 sprint-agnostic 한 전략 목표(initiative)이며, 산하 에픽이 sprint 를 소유합니다 (ADR-0014).",
            "inputSchema": {
                "type": "object",
                "required": ["title"],
                "properties": {
                    "title":       { "type": "string" },
                    "description": { "type": "string" },
                    "jira_key":    { "type": "string", "description": "Jira 이슈 키 (UNIQUE nullable)" }
                }
            }
        }),
        json!({
            "name": "mission_get",
            "description": "미션 단건 조회. id로 Mission을 반환합니다.",
            "inputSchema": {
                "type": "object",
                "required": ["id"],
                "properties": {
                    "id": { "type": "integer" }
                }
            }
        }),
        json!({
            "name": "mission_list",
            "description": "미션 목록 조회. 기본값: active only. include_completed=true 시 모든 상태 반환.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "status":             { "type": "string", "enum": MissionStatus::ALL },
                    "include_completed":  { "type": "boolean", "description": "true면 completed/cancelled 포함" }
                }
            }
        }),
        json!({
            "name": "mission_update",
            "description": "미션 수정. id 필수, 나머지 optional. status=completed/cancelled는 사용자 전용.",
            "inputSchema": {
                "type": "object",
                "required": ["id", "agent_id"],
                "properties": {
                    "id":          { "type": "integer" },
                    "title":       { "type": "string" },
                    "description": { "type": "string" },
                    "jira_key":    { "type": "string" },
                    "status":      { "type": "string", "enum": MissionStatus::ALL },
                    "agent_id":    { "type": "string" }
                }
            }
        }),
        json!({
            "name": "mission_delete",
            "description": "미션 삭제. 하위 에픽이 있으면 삭제 거부됩니다 (Validation error).",
            "inputSchema": {
                "type": "object",
                "required": ["id"],
                "properties": {
                    "id": { "type": "integer" }
                }
            }
        }),
        json!({
            "name": "mission_get_tree",
            "description": "미션의 계층 트리를 반환합니다. Mission → Epics → Issues 구조. 같은 미션 산하의 에픽이 서로 다른 sprint 에 속할 수 있습니다. id 또는 jira_key 중 하나 필수.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id":       { "type": "integer" },
                    "jira_key": { "type": "string", "description": "Jira 키로 미션 조회" }
                }
            }
        }),
    ]
}

pub async fn mission_create(db: Arc<Db>, args: &Value) -> Result<Value> {
    let title = args["title"]
        .as_str()
        .ok_or_else(|| Error::Validation("title required".into()))?;
    let input = CreateMissionInput {
        title: title.to_string(),
        description: args["description"].as_str().map(|s| s.to_string()),
        jira_key: args["jira_key"].as_str().map(|s| s.to_string()),
    };
    let m = db.mission_create(input).await?;
    Ok(serde_json::to_value(&m).unwrap())
}

pub async fn mission_get(db: Arc<Db>, args: &Value) -> Result<Value> {
    let id = args["id"]
        .as_i64()
        .ok_or_else(|| Error::Validation("id required".into()))?;
    let m = db.mission_get(id).await?;
    Ok(serde_json::to_value(&m).unwrap())
}

pub async fn mission_list(db: Arc<Db>, args: &Value) -> Result<Value> {
    let filter = MissionFilter {
        status: args["status"]
            .as_str()
            .and_then(|s| serde_json::from_value(json!(s)).ok()),
        include_completed: args["include_completed"].as_bool().unwrap_or(false),
    };
    let missions = db.mission_list(filter).await?;
    Ok(serde_json::to_value(&missions).unwrap())
}

pub async fn mission_update(db: Arc<Db>, args: &Value) -> Result<Value> {
    let id = args["id"]
        .as_i64()
        .ok_or_else(|| Error::Validation("id required".into()))?;
    let input = UpdateMissionInput {
        title: args["title"].as_str().map(|s| s.to_string()),
        description: args["description"].as_str().map(|s| s.to_string()),
        jira_key: args["jira_key"].as_str().map(|s| s.to_string()),
        status: args["status"]
            .as_str()
            .and_then(|s| serde_json::from_value(json!(s)).ok()),
    };
    let agent_id = args["agent_id"].as_str()
        .ok_or_else(|| Error::Validation("agent_id required".into()))?;
    let m = db.mission_update(id, input, agent_id).await?;
    Ok(serde_json::to_value(&m).unwrap())
}

pub async fn mission_delete(db: Arc<Db>, args: &Value) -> Result<Value> {
    let id = args["id"]
        .as_i64()
        .ok_or_else(|| Error::Validation("id required".into()))?;
    db.mission_delete(id).await?;
    Ok(json!({ "deleted": true, "id": id }))
}

pub async fn mission_get_tree(db: Arc<Db>, args: &Value) -> Result<Value> {
    let id = if let Some(id) = args["id"].as_i64() {
        id
    } else if let Some(jira_key) = args["jira_key"].as_str() {
        db.mission_get_by_jira_key(jira_key).await?.id
    } else {
        return Err(Error::Validation("id or jira_key required".into()));
    };

    let tree = db.mission_get_tree(id).await?;
    Ok(serde_json::to_value(&tree).unwrap())
}
