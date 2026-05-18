use engram_core::{Db, models::history::EntityType};
use serde_json::{json, Value};
use std::sync::Arc;

pub fn tool_definitions() -> Vec<Value> {
    vec![
        json!({ "name": "history_for",
            "description": "특정 엔티티의 변경 이력을 시간순(오름차순)으로 반환합니다. 멀티 에이전트 환경에서 '누가 언제 무엇을 바꿨는지' 추적용. ADR-0009 의 changed_by 가 의미 있는 식별자라면 어느 에이전트의 행동인지 식별 가능.",
            "inputSchema": { "type": "object", "required": ["entity_type", "entity_id"],
                "properties": {
                    "entity_type": { "type": "string", "enum": ["sprint","epic","issue","task","note"] },
                    "entity_id":   { "type": "integer" }
                }
            }
        }),
        json!({ "name": "history_by_agent",
            "description": "특정 에이전트(또는 'user')가 남긴 최근 변경 이력. 'A 에이전트가 오늘 어떤 작업을 했나' 식의 활동 모니터링용.",
            "inputSchema": { "type": "object", "required": ["agent_id"],
                "properties": {
                    "agent_id": { "type": "string", "description": "조회할 액터 식별자 (예: 'user', 'claude-opus@sess-abc')" },
                    "limit":    { "type": "integer", "description": "최대 반환 건수 (기본 50, 최대 500)", "default": 50 }
                }
            }
        }),
        json!({ "name": "history_recent",
            "description": "최근 N분 이내 또는 최근 limit 건의 모든 변경 이력 (cross-entity). 멀티 에이전트 환경의 실시간 활동 대시보드 / 사후 감사용.",
            "inputSchema": { "type": "object",
                "properties": {
                    "since_minutes": { "type": "integer", "description": "이 시간 안의 이력만. 생략 시 limit 기준만 적용." },
                    "limit":         { "type": "integer", "description": "최대 반환 건수 (기본 100, 최대 500)", "default": 100 }
                }
            }
        }),
    ]
}

pub async fn for_entity(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let entity_type: EntityType = args["entity_type"].as_str()
        .and_then(|s| serde_json::from_value(serde_json::Value::String(s.to_string())).ok())
        .ok_or_else(|| engram_core::Error::Validation("entity_type 필수 (sprint|epic|issue|task|note)".to_string()))?;
    let entity_id = args["entity_id"].as_i64()
        .ok_or_else(|| engram_core::Error::Validation("entity_id is required".to_string()))?;
    Ok(serde_json::to_value(db.history_list(entity_type, entity_id).await?).unwrap())
}

pub async fn by_agent(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let agent_id = args["agent_id"].as_str()
        .ok_or_else(|| engram_core::Error::Validation("agent_id is required".to_string()))?;
    let limit = args["limit"].as_i64().unwrap_or(50).clamp(1, 500);
    Ok(serde_json::to_value(db.history_by_agent(agent_id, limit).await?).unwrap())
}

pub async fn recent(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let limit = args["limit"].as_i64().unwrap_or(100).clamp(1, 500);
    let since_minutes = args["since_minutes"].as_i64();
    Ok(serde_json::to_value(db.history_recent(limit, since_minutes).await?).unwrap())
}
