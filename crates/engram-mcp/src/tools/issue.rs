use engram_core::{Db, models::issue::*};
use serde_json::{json, Value};
use std::sync::Arc;

pub fn tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "my_blocked_issues",
            "description": "현재 프로젝트의 블로킹 의존성 그래프를 반환합니다. 해소 가능한 리프 blocker와 체인 경로를 보여줍니다. 작업이 막혀있을 때 호출하세요.",
            "inputSchema": {
                "type": "object",
                "required": ["project_key"],
                "properties": {
                    "project_key": { "type": "string" }
                }
            }
        }),
        json!({ "name": "issue_create",
            "description": "새 이슈를 required(승인대기) 상태로 생성합니다. sprint_id 를 지정하면 해당 스프린트에, 생략하면 백로그에 들어갑니다. 작업 시작 전 반드시 사용자가 ready 로 승격해야 합니다.",
            "inputSchema": { "type": "object", "required": ["epic_id", "title"],
                "properties": {
                    "epic_id":     { "type": "integer" },
                    "sprint_id":   { "type": "integer", "description": "소속 스프린트 ID (생략 시 백로그)" },
                    "title":       { "type": "string" },
                    "description": { "type": "string" },
                    "priority":    { "type": "string", "enum": ["critical","high","medium","low"] }
                }
            }
        }),
        json!({ "name": "issue_set_sprint",
            "description": "이슈의 소속 스프린트를 변경합니다. sprint_id 를 생략하면 백로그로 이동합니다.",
            "inputSchema": { "type": "object", "required": ["id"],
                "properties": { "id": { "type": "integer" }, "sprint_id": { "type": "integer" } }
            }
        }),
        json!({ "name": "issue_get", "description": "이슈 상세를 조회합니다.",
            "inputSchema": { "type": "object", "required": ["id"],
                "properties": { "id": { "type": "integer" }, "include_tasks": { "type": "boolean" }, "include_notes": { "type": "boolean" } }
            }
        }),
        json!({ "name": "issue_list", "description": "이슈 목록을 조회합니다.",
            "inputSchema": { "type": "object",
                "properties": { "epic_id": { "type": "integer" }, "project_key": { "type": "string" }, "status": { "type": "string" } }
            }
        }),
        json!({ "name": "issue_update", "description": "이슈 상태/정보를 수정합니다. draft→approved 전환으로 작업 시작을 승인합니다.",
            "inputSchema": { "type": "object", "required": ["id"],
                "properties": { "id": { "type": "integer" }, "status": { "type": "string" }, "priority": { "type": "string" } }
            }
        }),
        json!({ "name": "issue_link",
            "description": "이슈 간 관계를 설정합니다. blocked_by 관계는 source=blocker, target=blocked, link_type=blocks로 설정하세요.",
            "inputSchema": { "type": "object", "required": ["source_id", "target_id", "link_type"],
                "properties": {
                    "source_id": { "type": "integer" },
                    "target_id": { "type": "integer" },
                    "link_type": { "type": "string", "enum": ["blocks","relates_to","duplicates"] }
                }
            }
        }),
        json!({ "name": "issue_unlink", "description": "이슈 간 관계를 제거합니다.",
            "inputSchema": { "type": "object", "required": ["link_id"],
                "properties": { "link_id": { "type": "integer" } }
            }
        }),
    ]
}

pub async fn create(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let priority: Option<IssuePriority> = args["priority"].as_str()
        .and_then(|s| serde_json::from_value(serde_json::Value::String(s.to_string())).ok());
    let input = CreateIssueInput {
        epic_id:     args["epic_id"].as_i64().unwrap_or(0),
        sprint_id:   args["sprint_id"].as_i64(),
        title:       args["title"].as_str().unwrap_or("").to_string(),
        description: args["description"].as_str().map(String::from),
        goal:        args["goal"].as_str().map(String::from),
        priority,
    };
    Ok(serde_json::to_value(db.issue_create(input).await?).unwrap())
}

pub async fn set_sprint(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let id = args["id"].as_i64()
        .ok_or_else(|| engram_core::Error::Validation("id is required".to_string()))?;
    let sprint_id = args["sprint_id"].as_i64(); // None → 백로그로 이동
    Ok(serde_json::to_value(db.issue_set_sprint(id, sprint_id, "agent").await?).unwrap())
}

pub async fn get(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let id = args["id"].as_i64().unwrap_or(0);
    Ok(serde_json::to_value(db.issue_get(id).await?).unwrap())
}

pub async fn list(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let filter = IssueFilter {
        epic_id:     args["epic_id"].as_i64(),
        project_key: args["project_key"].as_str().map(String::from),
        ..Default::default()
    };
    Ok(serde_json::to_value(db.issue_list(filter).await?).unwrap())
}

pub async fn update(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let id = args["id"].as_i64().unwrap_or(0);

    let status: Option<IssueStatus> = args["status"].as_str()
        .and_then(|s| serde_json::from_value(serde_json::Value::String(s.to_string())).ok());

    let priority: Option<IssuePriority> = args["priority"].as_str()
        .and_then(|s| serde_json::from_value(serde_json::Value::String(s.to_string())).ok());

    let title: Option<String> = args["title"].as_str().map(String::from);
    let description: Option<String> = args["description"].as_str().map(String::from);
    let goal: Option<String> = args["goal"].as_str().map(String::from);

    let input = UpdateIssueInput { status, priority, title, description, goal };
    Ok(serde_json::to_value(db.issue_update(id, input, "agent").await?).unwrap())
}

pub async fn link(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let source_id = args["source_id"].as_i64().unwrap_or(0);
    let target_id = args["target_id"].as_i64().unwrap_or(0);
    let link_type = match args["link_type"].as_str().unwrap_or("blocks") {
        "relates_to" => LinkType::RelatesTo,
        "duplicates" => LinkType::Duplicates,
        _            => LinkType::Blocks,
    };
    Ok(serde_json::to_value(db.issue_link(source_id, target_id, link_type).await?).unwrap())
}

pub async fn my_blocked_issues(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let project_key = args["project_key"].as_str().unwrap_or("");
    let graph = db.blocked_issues_graph(project_key).await?;
    Ok(serde_json::to_value(&graph).unwrap())
}

pub async fn unlink(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let link_id = args["link_id"].as_i64().unwrap_or(0);
    db.issue_unlink(link_id).await?;
    Ok(serde_json::json!({ "ok": true }))
}
