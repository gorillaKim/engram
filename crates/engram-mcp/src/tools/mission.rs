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
                    "include_completed":  { "type": "boolean", "description": "true면 completed/cancelled 포함" },
                    "project_key":        { "type": "string", "description": "특정 프로젝트 미션 필터" },
                    "sprint_id":          { "type": "integer", "description": "특정 스프린트 미션 필터" }
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
    let mode = super::get_mode(args);
    if matches!(mode, engram_core::models::OutputMode::Agent) {
        Ok(super::format::success(&format!(
            "Mission #{} created.",
            m.id
        )))
    } else {
        Ok(json!({ "id": m.id, "status": "ok" }))
    }
}

pub async fn mission_get(db: Arc<Db>, args: &Value) -> Result<Value> {
    let id = args["id"]
        .as_i64()
        .ok_or_else(|| Error::Validation("id required".into()))?;
    let m = db.mission_get(id).await?;
    let mode = super::get_mode(args);
    if matches!(mode, engram_core::models::OutputMode::Agent) {
        let status_str = serde_json::to_value(&m.status)
            .unwrap()
            .as_str()
            .unwrap_or("")
            .to_string();
        let fields = vec![
            ("ID", m.id.to_string()),
            ("Jira Key", m.jira_key.clone().unwrap_or_else(|| "-".to_string())),
            ("Title", m.title.clone()),
            ("Status", status_str),
            ("Description", m.description.clone().unwrap_or_else(|| "-".to_string())),
        ];
        Ok(Value::String(super::format::make_details("Mission Details", &fields)))
    } else {
        Ok(serde_json::to_value(&m).unwrap())
    }
}

pub async fn mission_list(db: Arc<Db>, args: &Value) -> Result<Value> {
    let filter = MissionFilter {
        status: args["status"]
            .as_str()
            .and_then(|s| serde_json::from_value(json!(s)).ok()),
        include_completed: args["include_completed"].as_bool().unwrap_or(false),
        project_key: args["project_key"].as_str().map(String::from),
        sprint_id: args["sprint_id"].as_i64(),
    };
    let missions = db.mission_list(filter).await?;
    let mode = super::get_mode(args);
    if matches!(mode, engram_core::models::OutputMode::Agent) {
        let headers = vec!["ID", "Jira Key", "Title", "Status"];
        let mut rows = Vec::new();
        for m in missions {
            let status_str = serde_json::to_value(&m.status)
                .unwrap()
                .as_str()
                .unwrap_or("")
                .to_string();
            rows.push(vec![
                m.id.to_string(),
                m.jira_key.unwrap_or_else(|| "-".to_string()),
                m.title,
                status_str,
            ]);
        }
        Ok(Value::String(super::format::make_table(&headers, &rows)))
    } else if matches!(mode, engram_core::models::OutputMode::Compact) {
        let mut val = serde_json::to_value(&missions).unwrap();
        if let Some(arr) = val.as_array_mut() {
            for m_val in arr {
                if let Some(obj) = m_val.as_object_mut() {
                    obj.remove("description");
                }
            }
        }
        Ok(val)
    } else {
        Ok(serde_json::to_value(&missions).unwrap())
    }
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
    db.mission_update(id, input, agent_id).await?;
    let mode = super::get_mode(args);
    if matches!(mode, engram_core::models::OutputMode::Agent) {
        Ok(super::format::success(&format!(
            "Mission #{} updated.",
            id
        )))
    } else {
        Ok(json!({ "id": id, "status": "ok" }))
    }
}

pub async fn mission_delete(db: Arc<Db>, args: &Value) -> Result<Value> {
    let id = args["id"]
        .as_i64()
        .ok_or_else(|| Error::Validation("id required".into()))?;
    db.mission_delete(id).await?;
    let mode = super::get_mode(args);
    if matches!(mode, engram_core::models::OutputMode::Agent) {
        Ok(super::format::success(&format!(
            "Mission #{} deleted.",
            id
        )))
    } else {
        Ok(json!({ "deleted": true, "id": id }))
    }
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
    let mode = super::get_mode(args);
    if matches!(mode, engram_core::models::OutputMode::Agent) {
        let mut out = format!("### Mission Tree: {} (ID: {})\n", tree.mission.title, tree.mission.id);
        let status_str = serde_json::to_value(&tree.mission.status)
            .unwrap()
            .as_str()
            .unwrap_or("")
            .to_string();
        out.push_str(&format!("- **Status**: {}\n", status_str));
        if let Some(ref desc) = tree.mission.description {
            out.push_str(&format!("- **Description**: {}\n", desc));
        }
        out.push_str("\n#### Sub-Epics & Issues:\n");
        if tree.epics.is_empty() {
            out.push_str("No epics assigned.\n");
        }
        for epic_with_issues in tree.epics {
            let epic = &epic_with_issues.epic;
            let epic_status = serde_json::to_value(&epic.status)
                .unwrap()
                .as_str()
                .unwrap_or("")
                .to_string();
            let sprint_str = epic
                .sprint_id
                .map(|sid| format!("Sprint #{}", sid))
                .unwrap_or_else(|| "Backlog".to_string());
            out.push_str(&format!(
                "- **Epic**: {} (ID: {}, Status: {}, Assignment: {})\n",
                epic.title, epic.id, epic_status, sprint_str
            ));
            for issue in epic_with_issues.issues {
                let issue_status = serde_json::to_value(&issue.status)
                    .unwrap()
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                out.push_str(&format!(
                    "  - **Issue #{}**: {} (Status: {})\n",
                    issue.id, issue.title, issue_status
                ));
            }
        }
        Ok(Value::String(out))
    } else if matches!(mode, engram_core::models::OutputMode::Compact) {
        let mut val = serde_json::to_value(&tree).unwrap();
        if let Some(obj) = val.as_object_mut() {
            if let Some(mission) = obj.get_mut("mission").and_then(|m| m.as_object_mut()) {
                mission.remove("description");
            }
            if let Some(epics) = obj.get_mut("epics").and_then(|e| e.as_array_mut()) {
                for ep_with_i in epics {
                    if let Some(ep_obj) = ep_with_i.as_object_mut() {
                        if let Some(epic) = ep_obj.get_mut("epic").and_then(|ep| ep.as_object_mut()) {
                            epic.remove("description");
                        }
                        if let Some(issues) = ep_obj.get_mut("issues").and_then(|i| i.as_array_mut()) {
                            for issue in issues {
                                if let Some(issue_obj) = issue.as_object_mut() {
                                    issue_obj.remove("description");
                                    issue_obj.remove("goal");
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(val)
    } else {
        Ok(serde_json::to_value(&tree).unwrap())
    }
}
