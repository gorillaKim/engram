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
            "description": "태스크의 테스트 체크리스트 전체를 조회합니다. checked/unchecked 상태를 포함합니다. task_id 또는 issue_id 중 최소 하나는 지정해야 합니다.",
            "inputSchema": { "type": "object",
                "properties": {
                    "task_id":  { "type": "integer", "description": "특정 태스크의 테스트 목록 조회" },
                    "issue_id": { "type": "integer", "description": "이슈 산하 모든 태스크의 테스트 목록 일괄 조회" }
                }
            }
        }),
        json!({ "name": "task_test_check",
            "description": "테스트 항목 하나를 완료 처리합니다. checked_at이 자동 기록됩니다.",
            "inputSchema": { "type": "object", "required": ["id", "agent_id"],
                "properties": {
                    "id":       { "type": "integer" },
                    "agent_id": { "type": "string", "description": "호출 액터 식별자" }
                }
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
            "inputSchema": { "type": "object", "required": ["id", "agent_id"],
                "properties": {
                    "id":       { "type": "integer" },
                    "agent_id": { "type": "string", "description": "호출 액터 식별자" }
                }
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
    let t = db.task_test_add(task_id, label).await?;
    let mode = super::get_mode(args);
    if matches!(mode, engram_core::models::OutputMode::Agent) {
        Ok(super::format::success(&format!("Task test #{} created.", t.id)))
    } else {
        Ok(serde_json::to_value(t).unwrap())
    }
}

pub async fn add_bulk(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let task_id = args["task_id"].as_i64().unwrap_or(0);
    let labels: Vec<String> = args["labels"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();
    let res = db.task_test_add_bulk(task_id, labels).await?;
    let mode = super::get_mode(args);
    if matches!(mode, engram_core::models::OutputMode::Agent) {
        Ok(super::format::success(&format!("Task tests created (Count: {}).", res.len())))
    } else {
        Ok(serde_json::to_value(res).unwrap())
    }
}

pub async fn list(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let task_id = args["task_id"].as_i64();
    let issue_id = args["issue_id"].as_i64();
    if task_id.is_none() && issue_id.is_none() {
        return Err(engram_core::Error::Validation("task_id 또는 issue_id 중 최소 하나는 필수입니다.".to_string()));
    }
    let list = db.task_test_list(task_id, issue_id).await?;
    let mode = super::get_mode(args);
    if matches!(mode, engram_core::models::OutputMode::Agent) {
        let headers = vec!["ID", "Task ID", "Label", "Status"];
        let mut rows = Vec::new();
        for t in list {
            let status = if t.checked_at.is_some() { "[x]" } else { "[ ]" };
            rows.push(vec![
                t.id.to_string(),
                t.task_id.to_string(),
                t.label,
                status.to_string(),
            ]);
        }
        Ok(Value::String(super::format::make_table(&headers, &rows)))
    } else {
        Ok(serde_json::to_value(list).unwrap())
    }
}

pub async fn check(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let agent_id = args["agent_id"].as_str()
        .ok_or_else(|| engram_core::Error::Validation("agent_id is required".to_string()))?;
    let id = args["id"].as_i64().unwrap_or(0);
    db.task_test_check(id, agent_id).await?;
    let mode = super::get_mode(args);
    if matches!(mode, engram_core::models::OutputMode::Agent) {
        Ok(super::format::success(&format!("Task test #{} checked.", id)))
    } else {
        Ok(json!({ "id": id, "status": "ok" }))
    }
}

pub async fn check_bulk(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let ids: Vec<i64> = args["ids"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|v| v.as_i64())
        .collect();
    db.task_test_check_bulk(ids.clone()).await?;
    let mode = super::get_mode(args);
    if matches!(mode, engram_core::models::OutputMode::Agent) {
        Ok(super::format::success(&format!("Task tests checked (Count: {}).", ids.len())))
    } else {
        Ok(json!({ "status": "ok" }))
    }
}

pub async fn uncheck(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let agent_id = args["agent_id"].as_str()
        .ok_or_else(|| engram_core::Error::Validation("agent_id is required".to_string()))?;
    let id = args["id"].as_i64().unwrap_or(0);
    db.task_test_uncheck(id, agent_id).await?;
    let mode = super::get_mode(args);
    if matches!(mode, engram_core::models::OutputMode::Agent) {
        Ok(super::format::success(&format!("Task test #{} unchecked.", id)))
    } else {
        Ok(json!({ "id": id, "status": "ok" }))
    }
}

pub async fn remove(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let id = args["id"].as_i64().unwrap_or(0);
    db.task_test_remove(id).await?;
    let mode = super::get_mode(args);
    if matches!(mode, engram_core::models::OutputMode::Agent) {
        Ok(super::format::success(&format!("Task test #{} deleted.", id)))
    } else {
        Ok(json!({ "ok": true }))
    }
}
