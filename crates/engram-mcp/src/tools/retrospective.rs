use std::sync::Arc;
use serde_json::{json, Value};
use engram_core::{
    models::retrospective::*,
    Db, Error, Result,
};

pub fn tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "retrospective_create",
            "description": "회고(Retrospective) 문서를 작성합니다. agent_id, projectKey, title, content가 필수이며 action_items 목록을 함께 전달할 수 있습니다.",
            "inputSchema": {
                "type": "object",
                "required": ["agent_id", "projectKey", "title", "content"],
                "properties": {
                    "agent_id": { "type": "string" },
                    "projectKey": { "type": "string" },
                    "title": { "type": "string" },
                    "content": { "type": "string" },
                    "sprintId": { "type": "integer" },
                    "missionId": { "type": "integer" },
                    "epicId": { "type": "integer" },
                    "actionItems": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "required": ["title"],
                            "properties": {
                                "title": { "type": "string" },
                                "description": { "type": "string" },
                                "linkedIssueId": { "type": "integer" },
                                "linkedNoteId": { "type": "integer" },
                                "ord": { "type": "number" }
                            }
                        }
                    }
                }
            }
        }),
        json!({
            "name": "retrospective_list",
            "description": "회고 목록을 조회합니다.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "projectKey": { "type": "string" },
                    "sprintId": { "type": "integer" },
                    "limit": { "type": "integer" }
                }
            }
        }),
        json!({
            "name": "retrospective_get",
            "description": "ID에 해당하는 회고 문서 및 포함된 액션 아이템 목록을 상세 조회합니다.",
            "inputSchema": {
                "type": "object",
                "required": ["id"],
                "properties": {
                    "id": { "type": "integer" }
                }
            }
        }),
        json!({
            "name": "retrospective_update",
            "description": "회고 문서를 수정합니다.",
            "inputSchema": {
                "type": "object",
                "required": ["id", "agent_id"],
                "properties": {
                    "id": { "type": "integer" },
                    "agent_id": { "type": "string" },
                    "title": { "type": "string" },
                    "content": { "type": "string" },
                    "sprintId": { "type": "integer" },
                    "missionId": { "type": "integer" },
                    "epicId": { "type": "integer" }
                }
            }
        }),
        json!({
            "name": "retrospective_delete",
            "description": "회고 문서를 삭제합니다.",
            "inputSchema": {
                "type": "object",
                "required": ["id", "agent_id"],
                "properties": {
                    "id": { "type": "integer" },
                    "agent_id": { "type": "string" }
                }
            }
        }),
        json!({
            "name": "retro_action_item_add",
            "description": "특정 회고에 새 액션 아이템을 추가합니다.",
            "inputSchema": {
                "type": "object",
                "required": ["agent_id", "retroId", "title"],
                "properties": {
                    "agent_id": { "type": "string" },
                    "retroId": { "type": "integer" },
                    "title": { "type": "string" },
                    "description": { "type": "string" },
                    "linkedIssueId": { "type": "integer" },
                    "linkedNoteId": { "type": "integer" },
                    "ord": { "type": "number" }
                }
            }
        }),
        json!({
            "name": "retro_action_item_update",
            "description": "액션 아이템 상태 또는 연결 정보를 수정합니다.",
            "inputSchema": {
                "type": "object",
                "required": ["agent_id", "id"],
                "properties": {
                    "id": { "type": "integer" },
                    "agent_id": { "type": "string" },
                    "title": { "type": "string" },
                    "description": { "type": "string" },
                    "status": { "type": "string" },
                    "linkedIssueId": { "type": "integer" },
                    "linkedNoteId": { "type": "integer" },
                    "ord": { "type": "number" }
                }
            }
        }),
        json!({
            "name": "retro_action_item_convert_to_issue",
            "description": "회고 액션 아이템 1개 또는 회고 전체 미연결 액션 아이템을 retro-{{스프린트이름}} 미션 하위 이슈로 전환합니다.",
            "inputSchema": {
                "type": "object",
                "required": ["agent_id"],
                "properties": {
                    "agent_id": { "type": "string" },
                    "id": { "type": "integer", "description": "단일 액션 아이템 ID" },
                    "retroId": { "type": "integer", "description": "회고 전체 일괄 전환 시 회고 ID" }
                }
            }
        }),
    ]
}

pub async fn retrospective_create(db: Arc<Db>, args: &Value) -> Result<Value> {
    let agent_id = args["agent_id"].as_str().ok_or_else(|| Error::Validation("agent_id is required".into()))?;
    let project_key = args["projectKey"].as_str().ok_or_else(|| Error::Validation("projectKey is required".into()))?;
    let title = args["title"].as_str().ok_or_else(|| Error::Validation("title is required".into()))?;
    let content = args["content"].as_str().ok_or_else(|| Error::Validation("content is required".into()))?;

    let action_items = if let Some(arr) = args["actionItems"].as_array() {
        let mut items = Vec::new();
        for val in arr {
            let item_title = val["title"].as_str().ok_or_else(|| Error::Validation("actionItems item title is required".into()))?;
            items.push(CreateRetroActionItemInput {
                title: item_title.to_string(),
                description: val["description"].as_str().map(|s| s.to_string()),
                linked_issue_id: val["linkedIssueId"].as_i64(),
                linked_note_id: val["linkedNoteId"].as_i64(),
                ord: val["ord"].as_f64(),
            });
        }
        Some(items)
    } else {
        None
    };

    let input = CreateRetrospectiveInput {
        project_key: project_key.to_string(),
        title: title.to_string(),
        content: content.to_string(),
        sprint_id: args["sprintId"].as_i64(),
        mission_id: args["missionId"].as_i64(),
        epic_id: args["epicId"].as_i64(),
        agent_id: Some(agent_id.to_string()),
        action_items,
    };

    let res = db.retrospective_create(input).await?;
    Ok(serde_json::to_value(res).unwrap())
}

pub async fn retrospective_list(db: Arc<Db>, args: &Value) -> Result<Value> {
    let project_key = args["projectKey"].as_str();
    let sprint_id = args["sprintId"].as_i64();
    let limit = args["limit"].as_u64().unwrap_or(50) as u32;

    let res = db.retrospective_list(project_key, sprint_id, limit).await?;
    Ok(serde_json::to_value(res).unwrap())
}

pub async fn retrospective_get(db: Arc<Db>, args: &Value) -> Result<Value> {
    let id = args["id"].as_i64().ok_or_else(|| Error::Validation("id is required".into()))?;
    let res = db.retrospective_get(id).await?;
    Ok(serde_json::to_value(res).unwrap())
}

pub async fn retrospective_update(db: Arc<Db>, args: &Value) -> Result<Value> {
    let id = args["id"].as_i64().ok_or_else(|| Error::Validation("id is required".into()))?;
    let agent_id = args["agent_id"].as_str().ok_or_else(|| Error::Validation("agent_id is required".into()))?;

    let input = UpdateRetrospectiveInput {
        title: args["title"].as_str().map(|s| s.to_string()),
        content: args["content"].as_str().map(|s| s.to_string()),
        sprint_id: args["sprintId"].as_i64(),
        mission_id: args["missionId"].as_i64(),
        epic_id: args["epicId"].as_i64(),
    };

    let res = db.retrospective_update(id, input, Some(agent_id)).await?;
    Ok(serde_json::to_value(res).unwrap())
}

pub async fn retrospective_delete(db: Arc<Db>, args: &Value) -> Result<Value> {
    let id = args["id"].as_i64().ok_or_else(|| Error::Validation("id is required".into()))?;
    let _agent_id = args["agent_id"].as_str().ok_or_else(|| Error::Validation("agent_id is required".into()))?;

    db.retrospective_delete(id).await?;
    Ok(json!({ "success": true, "deleted_id": id }))
}

pub async fn retro_action_item_add(db: Arc<Db>, args: &Value) -> Result<Value> {
    let _agent_id = args["agent_id"].as_str().ok_or_else(|| Error::Validation("agent_id is required".into()))?;
    let retro_id = args["retroId"].as_i64().ok_or_else(|| Error::Validation("retroId is required".into()))?;
    let title = args["title"].as_str().ok_or_else(|| Error::Validation("title is required".into()))?;

    let input = CreateRetroActionItemInput {
        title: title.to_string(),
        description: args["description"].as_str().map(|s| s.to_string()),
        linked_issue_id: args["linkedIssueId"].as_i64(),
        linked_note_id: args["linkedNoteId"].as_i64(),
        ord: args["ord"].as_f64(),
    };

    let res = db.retro_action_item_create(retro_id, input).await?;
    Ok(serde_json::to_value(res).unwrap())
}

pub async fn retro_action_item_update(db: Arc<Db>, args: &Value) -> Result<Value> {
    let id = args["id"].as_i64().ok_or_else(|| Error::Validation("id is required".into()))?;
    let _agent_id = args["agent_id"].as_str().ok_or_else(|| Error::Validation("agent_id is required".into()))?;

    let input = UpdateRetroActionItemInput {
        title: args["title"].as_str().map(|s| s.to_string()),
        description: args["description"].as_str().map(|s| s.to_string()),
        status: args["status"].as_str().map(|s| s.to_string()),
        linked_issue_id: args["linkedIssueId"].as_i64(),
        linked_note_id: args["linkedNoteId"].as_i64(),
        ord: args["ord"].as_f64(),
    };

    let res = db.retro_action_item_update(id, input).await?;
    Ok(serde_json::to_value(res).unwrap())
}

pub async fn retro_action_item_convert_to_issue(db: Arc<Db>, args: &Value) -> Result<Value> {
    let agent_id = args["agent_id"].as_str().ok_or_else(|| Error::Validation("agent_id is required".into()))?;

    if let Some(id) = args["id"].as_i64() {
        let issue = db.retro_action_item_convert_to_issue(id, Some(agent_id)).await?;
        Ok(serde_json::to_value(issue).unwrap())
    } else if let Some(retro_id) = args["retroId"].as_i64() {
        let issues = db.retrospective_bulk_convert_actions_to_issues(retro_id, Some(agent_id)).await?;
        Ok(serde_json::to_value(issues).unwrap())
    } else {
        Err(Error::Validation("Either id or retroId must be provided".into()))
    }
}
