use engram_core::{Db, models::sprint::*};
use serde_json::{json, Value};
use std::sync::Arc;

pub fn tool_definitions() -> Vec<Value> {
    vec![
        json!({ "name": "sprint_create", "description": "새 스프린트를 생성합니다.",
            "inputSchema": { "type": "object", "required": ["name"],
                "properties": {
                    "name":  { "type": "string" }, "goal": { "type": "string" },
                    "start_date": { "type": "string" }, "end_date": { "type": "string" }
                }
            }
        }),
        json!({ "name": "sprint_list", "description": "스프린트 목록을 조회합니다.",
            "inputSchema": { "type": "object", "properties": { "status_filter": { "type": "string" } } }
        }),
        json!({ "name": "sprint_current", "description": "현재 활성 스프린트를 조회합니다.",
            "inputSchema": { "type": "object" }
        }),
        json!({ "name": "sprint_update", "description": "스프린트 정보를 수정합니다.",
            "inputSchema": { "type": "object", "required": ["id"],
                "properties": { "id": { "type": "integer" }, "status": { "type": "string" }, "goal": { "type": "string" } }
            }
        }),
    ]
}

pub async fn create(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let input = CreateSprintInput {
        name: args["name"].as_str().unwrap_or("").to_string(),
        goal: args["goal"].as_str().map(String::from),
        start_date: args["start_date"].as_str().map(String::from),
        end_date: args["end_date"].as_str().map(String::from),
    };
    Ok(serde_json::to_value(db.sprint_create(input).await?).unwrap())
}

pub async fn list(db: Arc<Db>, _args: &Value) -> engram_core::Result<Value> {
    Ok(serde_json::to_value(db.sprint_list(None).await?).unwrap())
}

pub async fn current(db: Arc<Db>, _args: &Value) -> engram_core::Result<Value> {
    Ok(serde_json::to_value(db.sprint_current().await?).unwrap())
}

pub async fn update(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let id = args["id"].as_i64().unwrap_or(0);
    let status: Option<SprintStatus> = args["status"].as_str()
        .and_then(|s| serde_json::from_value(Value::String(s.to_string())).ok());
    let input = UpdateSprintInput {
        name:       args["name"].as_str().map(String::from),
        goal:       args["goal"].as_str().map(String::from),
        status,
        start_date: args["start_date"].as_str().map(String::from),
        end_date:   args["end_date"].as_str().map(String::from),
    };
    Ok(serde_json::to_value(db.sprint_update(id, input).await?).unwrap())
}
