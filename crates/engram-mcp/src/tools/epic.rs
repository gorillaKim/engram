use engram_core::{Db, models::epic::*};
use serde_json::{json, Value};
use std::sync::Arc;

pub fn tool_definitions() -> Vec<Value> {
    vec![
        json!({ "name": "epic_create", "description": "새 에픽을 생성합니다. mission_id 로 소속 미션을 지정하세요 (필수). 에픽은 프로젝트(project_key) 단위 카테고리 — sprint 와 무관합니다.",
            "inputSchema": { "type": "object", "required": ["project_key", "title", "mission_id"],
                "properties": {
                    "project_key":  { "type": "string",  "description": "프로젝트 식별자 (예: 'xpert-da-web')" },
                    "mission_id":   { "type": "integer", "description": "소속 미션 ID. 지정하면 이 에픽 하위 이슈가 mission_id 를 자동 상속합니다." },
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
        json!({ "name": "epic_list", "description": "에픽 목록을 조회합니다. 기본값은 completed 상태를 제외한 active/cancelled 에픽만 반환합니다. 완료 에픽 포함 시 include_completed=true 를 사용하세요.",
            "inputSchema": { "type": "object",
                "properties": {
                    "project_key":       { "type": "string" },
                    "include_completed": { "type": "boolean", "description": "기본 false. true 시 completed 에픽도 포함하여 반환" }
                }
            }
        }),
        json!({ "name": "epic_update", "description": "에픽 정보(제목/설명/상태/미션) 를 수정합니다. mission_id 변경 시 cascade_issues=true(기본)이면 하위 이슈 mission_id 도 함께 갱신됩니다.",
            "inputSchema": { "type": "object", "required": ["id"],
                "properties": {
                    "id":              { "type": "integer" },
                    "title":           { "type": "string" },
                    "description":     { "type": "string" },
                    "status":          { "type": "string" },
                    "mission_id":      { "type": "integer", "description": "미션 변경 (cascade_issues=true이면 하위 이슈도 함께 변경)" },
                    "cascade_issues":  { "type": "boolean", "description": "true(기본): 하위 이슈 mission_id도 함께 변경" },
                    "agent_id":        { "type": "string" }
                }
            }
        }),
        json!({ "name": "epic_delete", "description": "에픽을 삭제합니다. 하위 이슈/태스크/노트/링크가 함께 cascade 삭제됩니다 — 비가역 작업이므로 신중하게 호출하세요.",
            "inputSchema": { "type": "object", "required": ["id"],
                "properties": {
                    "id":       { "type": "integer" },
                    "agent_id": { "type": "string", "description": "호출 액터 식별자 (예: 'user', 'claude-opus@sess-abc'). 생략 시 'agent'." }
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
        title:       args["title"].as_str().unwrap_or("").to_string(),
        description: args["description"].as_str().map(String::from),
    };
    Ok(serde_json::to_value(db.epic_create(input).await?).unwrap())
}

pub async fn delete(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let id = args["id"].as_i64()
        .ok_or_else(|| engram_core::Error::Validation("id is required".to_string()))?;
    let agent_id = args["agent_id"].as_str().unwrap_or("agent");
    db.epic_delete(id, agent_id).await?;
    Ok(json!({ "ok": true, "deleted_id": id }))
}

pub async fn get(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let id = args["id"].as_i64().unwrap_or(0);
    Ok(serde_json::to_value(db.epic_get(id).await?).unwrap())
}

pub async fn list(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let project_key = args["project_key"].as_str();
    let include_completed = args["include_completed"].as_bool().unwrap_or(false);
    Ok(serde_json::to_value(db.epic_list(project_key, include_completed).await?).unwrap())
}

pub async fn update(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let id = args["id"].as_i64().unwrap_or(0);
    let status: Option<EpicStatus> = args["status"].as_str()
        .and_then(|s| serde_json::from_value(Value::String(s.to_string())).ok());
    let agent_id = args["agent_id"].as_str().unwrap_or("agent");
    let has_cascade = args["mission_id"].as_i64().is_some()
        && args["cascade_issues"].as_bool().unwrap_or(true);
    let input = UpdateEpicInput {
        title:           args["title"].as_str().map(String::from),
        description:     args["description"].as_str().map(String::from),
        status,
        mission_id:      args["mission_id"].as_i64(),
        cascade_issues:  args["cascade_issues"].as_bool().unwrap_or(true),
    };
    let (epic, cascade_updated, cascade_skipped) = db.epic_update(id, input, agent_id).await?;
    let mut result = serde_json::to_value(&epic).unwrap();
    if has_cascade {
        result = json!({
            "epic": epic,
            "cascade_updated": cascade_updated,
            "cascade_skipped": cascade_skipped,
        });
    }
    Ok(result)
}
