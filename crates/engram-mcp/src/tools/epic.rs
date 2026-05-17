use engram_core::{Db, models::epic::*};
use serde_json::{json, Value};
use std::sync::Arc;

pub fn tool_definitions() -> Vec<Value> {
    vec![
        json!({ "name": "epic_create", "description": "새 에픽을 생성합니다. 에픽은 프로젝트(project_key) 단위 카테고리 — sprint 와 무관합니다. 이슈에 sprint_id 를 지정해 스프린트로 끌어옵니다.",
            "inputSchema": { "type": "object", "required": ["project_key", "title"],
                "properties": {
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
        json!({ "name": "epic_list", "description": "에픽 목록을 조회합니다. project_key 로 필터링 가능합니다.",
            "inputSchema": { "type": "object",
                "properties": {
                    "project_key": { "type": "string" },
                    "status":      { "type": "string" }
                }
            }
        }),
        json!({ "name": "epic_update", "description": "에픽 정보(제목/설명/상태) 를 수정합니다.",
            "inputSchema": { "type": "object", "required": ["id"],
                "properties": {
                    "id": { "type": "integer" },
                    "title": { "type": "string" },
                    "description": { "type": "string" },
                    "status": { "type": "string" }
                }
            }
        }),
        json!({ "name": "epic_delete", "description": "에픽을 삭제합니다. 이슈가 하나라도 연결된 에픽은 삭제할 수 없습니다 — 먼저 이슈를 옮기거나 삭제하세요.",
            "inputSchema": { "type": "object", "required": ["id"],
                "properties": { "id": { "type": "integer" } }
            }
        }),
    ]
}

pub async fn create(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let input = CreateEpicInput {
        project_key: args["project_key"].as_str().unwrap_or("").to_string(),
        title:       args["title"].as_str().unwrap_or("").to_string(),
        description: args["description"].as_str().map(String::from),
    };
    Ok(serde_json::to_value(db.epic_create(input).await?).unwrap())
}

pub async fn delete(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let id = args["id"].as_i64()
        .ok_or_else(|| engram_core::Error::Validation("id is required".to_string()))?;
    db.epic_delete(id).await?;
    Ok(json!({ "ok": true, "deleted_id": id }))
}

pub async fn get(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let id = args["id"].as_i64().unwrap_or(0);
    Ok(serde_json::to_value(db.epic_get(id).await?).unwrap())
}

pub async fn list(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let project_key = args["project_key"].as_str();
    Ok(serde_json::to_value(db.epic_list(project_key, None).await?).unwrap())
}

pub async fn update(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let id = args["id"].as_i64().unwrap_or(0);
    let status: Option<EpicStatus> = args["status"].as_str()
        .and_then(|s| serde_json::from_value(Value::String(s.to_string())).ok());
    let input = UpdateEpicInput {
        title:       args["title"].as_str().map(String::from),
        description: args["description"].as_str().map(String::from),
        status,
    };
    Ok(serde_json::to_value(db.epic_update(id, input, "agent").await?).unwrap())
}

pub async fn list_backlog(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let project_key = args["project_key"].as_str();
    Ok(serde_json::to_value(db.epic_list(project_key, None).await?).unwrap())
}

pub async fn set_sprint(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    use engram_core::models::issue::IssueFilter;
    let id = args["id"].as_i64()
        .ok_or_else(|| engram_core::Error::Validation("id is required".to_string()))?;
    let sprint_id = args["sprint_id"].as_i64();
    let issues = db.issue_list(IssueFilter { epic_id: Some(id), ..Default::default() }).await?;
    for issue in &issues {
        db.issue_set_sprint(issue.id, sprint_id, "agent").await?;
    }
    Ok(serde_json::to_value(db.epic_get(id).await?).unwrap())
}
