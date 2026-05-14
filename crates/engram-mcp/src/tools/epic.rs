use engram_core::{Db, models::epic::*};
use serde_json::{json, Value};
use std::sync::Arc;

pub fn tool_definitions() -> Vec<Value> {
    vec![
        json!({ "name": "epic_create", "description": "새 에픽을 생성합니다. 에픽은 프로젝트(project_key)와 스프린트를 연결하는 작업 묶음입니다.",
            "inputSchema": { "type": "object", "required": ["sprint_id", "project_key", "title"],
                "properties": {
                    "sprint_id":    { "type": "integer", "description": "소속 스프린트 ID" },
                    "project_key":  { "type": "string",  "description": "프로젝트 식별자 (예: 'xpert-da-web')" },
                    "title":        { "type": "string" },
                    "description":  { "type": "string" }
                }
            }
        }),
        json!({ "name": "epic_get", "description": "에픽 상세를 조회합니다.",
            "inputSchema": { "type": "object", "required": ["id"],
                "properties": { "id": { "type": "integer" } }
            }
        }),
        json!({ "name": "epic_list", "description": "에픽 목록을 조회합니다. project_key로 필터링 가능합니다.",
            "inputSchema": { "type": "object",
                "properties": {
                    "sprint_id":   { "type": "integer" },
                    "project_key": { "type": "string" },
                    "status":      { "type": "string" }
                }
            }
        }),
        json!({ "name": "epic_update", "description": "에픽 정보를 수정합니다.",
            "inputSchema": { "type": "object", "required": ["id"],
                "properties": { "id": { "type": "integer" }, "title": { "type": "string" }, "status": { "type": "string" } }
            }
        }),
    ]
}

pub async fn create(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let input = CreateEpicInput {
        sprint_id:   args["sprint_id"].as_i64().unwrap_or(0),
        project_key: args["project_key"].as_str().unwrap_or("").to_string(),
        title:       args["title"].as_str().unwrap_or("").to_string(),
        description: args["description"].as_str().map(String::from),
    };
    Ok(serde_json::to_value(db.epic_create(input).await?).unwrap())
}

pub async fn get(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let id = args["id"].as_i64().unwrap_or(0);
    Ok(serde_json::to_value(db.epic_get(id).await?).unwrap())
}

pub async fn list(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let sprint_id   = args["sprint_id"].as_i64();
    let project_key = args["project_key"].as_str();
    Ok(serde_json::to_value(db.epic_list(sprint_id, project_key, None).await?).unwrap())
}

pub async fn update(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let id = args["id"].as_i64().unwrap_or(0);
    Ok(serde_json::to_value(db.epic_update(id, UpdateEpicInput::default()).await?).unwrap())
}
