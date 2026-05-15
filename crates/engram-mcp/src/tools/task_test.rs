use engram_core::Db;
use serde_json::{json, Value};
use std::sync::Arc;

pub fn tool_definitions() -> Vec<Value> {
    vec![
        json!({ "name": "task_test_add",
            "description": "태스크에 검증할 테스트 항목을 한 줄 추가합니다. 여러 항목을 한꺼번에 넣을 때는 task_test_add_bulk를 사용하세요.",
            "inputSchema": { "type": "object", "required": ["task_id", "label"],
                "properties": {
                    "task_id": { "type": "integer" },
                    "label":   { "type": "string" }
                }
            }
        }),
        json!({ "name": "task_test_add_bulk",
            "description": "태스크에 검증 항목을 여러 개 한꺼번에 추가합니다. 작업 시작 시 예상 테스트 목록을 한 번에 등록할 때 사용하세요.",
            "inputSchema": { "type": "object", "required": ["task_id", "labels"],
                "properties": {
                    "task_id": { "type": "integer" },
                    "labels":  { "type": "array", "items": { "type": "string" } }
                }
            }
        }),
        json!({ "name": "task_test_list",
            "description": "태스크의 테스트 체크리스트 전체를 조회합니다. checked/unchecked 상태를 포함합니다.",
            "inputSchema": { "type": "object", "required": ["task_id"],
                "properties": { "task_id": { "type": "integer" } }
            }
        }),
        json!({ "name": "task_test_check",
            "description": "테스트 항목 하나를 완료 처리합니다. checked_at이 자동 기록됩니다.",
            "inputSchema": { "type": "object", "required": ["id"],
                "properties": { "id": { "type": "integer" } }
            }
        }),
        json!({ "name": "task_test_check_bulk",
            "description": "테스트 항목 여러 개를 한 번에 완료 처리합니다. 일괄 검증 완료 시 사용하세요.",
            "inputSchema": { "type": "object", "required": ["ids"],
                "properties": {
                    "ids": { "type": "array", "items": { "type": "integer" } }
                }
            }
        }),
        json!({ "name": "task_test_uncheck",
            "description": "완료 처리된 테스트 항목을 미완료 상태로 되돌립니다. 재검증이 필요할 때 사용하세요.",
            "inputSchema": { "type": "object", "required": ["id"],
                "properties": { "id": { "type": "integer" } }
            }
        }),
        json!({ "name": "task_test_remove",
            "description": "테스트 항목을 목록에서 삭제합니다. 불필요해진 항목을 제거할 때 사용하세요.",
            "inputSchema": { "type": "object", "required": ["id"],
                "properties": { "id": { "type": "integer" } }
            }
        }),
    ]
}

pub async fn add(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let task_id = args["task_id"].as_i64().unwrap_or(0);
    let label   = args["label"].as_str().unwrap_or("").to_string();
    Ok(serde_json::to_value(db.task_test_add(task_id, label).await?).unwrap())
}

pub async fn add_bulk(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let task_id = args["task_id"].as_i64().unwrap_or(0);
    let labels: Vec<String> = args["labels"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();
    Ok(serde_json::to_value(db.task_test_add_bulk(task_id, labels).await?).unwrap())
}

pub async fn list(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let task_id = args["task_id"].as_i64().unwrap_or(0);
    Ok(serde_json::to_value(db.task_test_list(task_id).await?).unwrap())
}

pub async fn check(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let id = args["id"].as_i64().unwrap_or(0);
    Ok(serde_json::to_value(db.task_test_check(id).await?).unwrap())
}

pub async fn check_bulk(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let ids: Vec<i64> = args["ids"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|v| v.as_i64())
        .collect();
    Ok(serde_json::to_value(db.task_test_check_bulk(ids).await?).unwrap())
}

pub async fn uncheck(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let id = args["id"].as_i64().unwrap_or(0);
    Ok(serde_json::to_value(db.task_test_uncheck(id).await?).unwrap())
}

pub async fn remove(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let id = args["id"].as_i64().unwrap_or(0);
    db.task_test_remove(id).await?;
    Ok(json!({ "ok": true }))
}
