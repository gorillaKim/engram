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
            "description": "새 이슈를 required(승인대기) 상태로 생성합니다. sprint_id 와 mission_id 는 부모 epic 에서 자동 derive 되므로 직접 지정할 수 없습니다 (ADR-0014). 작업 시작 전 반드시 사용자가 ready 로 승격해야 합니다.",
            "inputSchema": { "type": "object", "required": ["epic_id", "title"],
                "properties": {
                    "epic_id":     { "type": "integer" },
                    "title":       { "type": "string" },
                    "description": { "type": "string" },
                    "goal":        { "type": "string", "description": "이슈의 성공 목표" },
                    "priority":    { "type": "string", "enum": ["critical","high","medium","low"] }
                }
            }
        }),
        json!({ "name": "issue_get", "description": "이슈 상세를 조회합니다.",
            "inputSchema": { "type": "object", "required": ["id"],
                "properties": {
                    "id": { "type": "integer" },
                    "include_tasks": { "type": "boolean" },
                    "include_notes": { "type": "boolean" },
                    "compact": { "type": "boolean", "description": "true인 경우 description 과 goal 을 NULL로 채워서 반환 (기본값 false)" }
                }
            }
        }),
        json!({ "name": "issue_list",
            "description": "이슈 목록을 조회합니다. sprint_id/project_key/status 필터를 조합해 활성 스프린트의 ready 이슈 큐 등을 만들 수 있습니다.",
            "inputSchema": { "type": "object",
                "properties": {
                    "epic_id":     { "type": "integer" },
                    "mission_id":  { "type": "integer", "description": "특정 미션 소속 이슈만 필터링" },
                    "sprint_id":   { "type": "integer" },
                    "project_key": { "type": "string" },
                    "status":      {
                        "oneOf": [
                            { "type": "string", "enum": IssueStatus::ALL },
                            { "type": "array", "items": { "type": "string", "enum": IssueStatus::ALL } }
                        ]
                    },
                    "backlog_only":{ "type": "boolean" }
                }
            }
        }),
        json!({ "name": "stalled_issues",
            "description": "지정 상태(기본 working)에서 threshold_minutes 이상 머문 이슈 목록을 반환합니다. 리더 에이전트가 정체된 작업을 단일 호출로 발견할 때 사용하세요. 반환값에는 entered_status_at, minutes_in_status 가 포함됩니다.",
            "inputSchema": { "type": "object", "required": ["threshold_minutes"],
                "properties": {
                    "project_key":       { "type": "string" },
                    "status":            { "type": "string", "enum": IssueStatus::ALL, "default": "working" },
                    "threshold_minutes": { "type": "integer", "minimum": 1 }
                }
            }
        }),
        json!({ "name": "issue_update",
            "description": "이슈 상태/정보를 수정합니다. agent_id 는 필수이며, history.changed_by 로 저장되어 멀티 에이전트 감사가 가능합니다. epic_id 를 보내면 이슈를 다른 에픽으로 이동(sprint/mission 자동 상속).",
            "inputSchema": { "type": "object", "required": ["id", "agent_id"],
                "properties": {
                    "id":          { "type": "integer" },
                    "status":      { "type": "string", "enum": ["required","ready","working","demo"] },
                    "priority":    { "type": "string", "enum": ["critical","high","medium","low"] },
                    "title":       { "type": "string" },
                    "description": { "type": "string" },
                    "goal":        { "type": "string" },
                    "epic_id":     { "type": "integer", "description": "다른 에픽으로 이동" },
                    "agent_id":    { "type": "string", "description": "호출 액터 식별자 (예: 'user', 'claude-opus@sess-abc')" }
                }
            }
        }),
        json!({ "name": "issue_link",
            "description": "이슈 간 관계를 설정합니다. blocked_by 관계는 source=blocker, target=blocked, link_type=blocks로 설정하세요.",
            "inputSchema": { "type": "object", "required": ["source_id", "target_id", "link_type", "agent_id"],
                "properties": {
                    "source_id": { "type": "integer" },
                    "target_id": { "type": "integer" },
                    "link_type": { "type": "string", "enum": ["blocks","relates_to","duplicates"] },
                    "agent_id":  { "type": "string", "description": "호출 액터 식별자" }
                }
            }
        }),
        json!({ "name": "issue_unlink", "description": "이슈 간 관계를 제거합니다.",
            "inputSchema": { "type": "object", "required": ["link_id", "agent_id"],
                "properties": {
                    "link_id":  { "type": "integer" },
                    "agent_id": { "type": "string", "description": "호출 액터 식별자" }
                }
            }
        }),
        json!({ "name": "issue_delete",
            "description": "이슈를 삭제합니다. 하위 태스크/노트/링크가 함께 cascade 삭제되며 비가역입니다. agent_id 를 명시하면 history.changed_by 에 그대로 기록됩니다 (생략 시 'agent').",
            "inputSchema": { "type": "object", "required": ["id"],
                "properties": {
                    "id":       { "type": "integer" },
                    "agent_id": { "type": "string", "description": "호출 액터 식별자. 'user' 면 사용자 액션, 그 외는 agent 식별자." }
                }
            }
        }),
        json!({ "name": "issue_claim",
            "description": "이슈를 working 상태로 점유합니다 (CAS, 멀티 에이전트 안전). 다른 에이전트가 이미 점유 중이면 거부됩니다. 작업 시작 직전 반드시 호출하세요. agent_id 는 필수 — 자기 식별자를 지정해야 release 시 권한 확인이 됩니다.",
            "inputSchema": { "type": "object", "required": ["id", "agent_id"],
                "properties": {
                    "id":       { "type": "integer" },
                    "agent_id": { "type": "string", "description": "점유할 에이전트 식별자 (예: 'claude-opus@sess-abc')." }
                }
            }
        }),
        json!({ "name": "issue_release",
            "description": "점유한 이슈를 해제하고 지정 상태로 전이합니다. 보통 ready (다른 에이전트가 픽업 가능) 또는 demo (사용자 검토 대기) 로 전이합니다. 기본은 자기가 잡은 이슈만 해제 가능. force=true 면 ownership 검증 우회 — 좀비 lease 회수용 (사용자 또는 리더가 stalled 에이전트의 점유를 강제 해제할 때만 사용). force 회수도 history 에 호출자 agent_id 로 감사 기록됩니다.",
            "inputSchema": { "type": "object", "required": ["id", "agent_id", "transition_to"],
                "properties": {
                    "id":            { "type": "integer" },
                    "agent_id":      { "type": "string" },
                    "transition_to": { "type": "string", "enum": ["ready","demo","required"] },
                    "force":         { "type": "boolean", "description": "기본 false. true 면 ownership 검증 우회. 좀비 lease 회수 시만 사용." }
                }
            }
        }),
        json!({
            "name": "issue_finish",
            "description": "demo 상태의 이슈를 finished 로 전이합니다. 사용자 전용 도구입니다 (agent_id 가 'user' 가 아니면 거부됩니다).",
            "inputSchema": {
                "type": "object",
                "required": ["id", "agent_id"],
                "properties": {
                    "id":       { "type": "integer" },
                    "agent_id": { "type": "string", "description": "호출자 식별자. 'user' 필수." }
                }
            }
        }),
        json!({
            "name": "issue_cancel",
            "description": "이슈를 cancelled 로 전이합니다. 사용자 전용 도구입니다 (agent_id 가 'user' 가 아니면 거부됩니다).",
            "inputSchema": {
                "type": "object",
                "required": ["id", "reason", "agent_id"],
                "properties": {
                    "id":       { "type": "integer" },
                    "reason":   { "type": "string", "description": "취소 사유" },
                    "agent_id": { "type": "string", "description": "호출자 식별자. 'user' 필수." }
                }
            }
        }),
        json!({
            "name": "planning_review_queue",
            "description": "지정 프로젝트의 기획 검토 대상 이슈 목록을 반환합니다. sprint_id 미지정 시 현재 활성 스프린트의 이슈들을 가져옵니다. statuses 필터로 특정 상태의 이슈만 필터링할 수 있습니다.",
            "inputSchema": {
                "type": "object",
                "required": ["project_key"],
                "properties": {
                    "project_key": { "type": "string" },
                    "sprint_id":   { "type": "integer" },
                    "statuses":    {
                        "type": "array",
                        "items": { "type": "string", "enum": IssueStatus::ALL }
                    }
                }
            }
        }),
        json!({
            "name": "issue_bulk_update",
            "description": "여러 이슈의 상태나 우선순위를 일괄 수정합니다. 부분 실패 시 실패한 이슈와 에러 내용을 함께 반환합니다.",
            "inputSchema": {
                "type": "object",
                "required": ["ids", "agent_id"],
                "properties": {
                    "ids": {
                        "type": "array",
                        "items": { "type": "integer" }
                    },
                    "status": {
                        "type": "string",
                        "enum": ["required", "ready", "working", "demo"]
                    },
                    "priority": {
                        "type": "string",
                        "enum": ["critical", "high", "medium", "low"]
                    },
                    "agent_id": {
                        "type": "string",
                        "description": "호출 액터 식별자"
                    }
                }
            }
        }),
    ]
}

pub async fn create(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    if args["sprint_id"].is_number() || args["mission_id"].is_number() {
        return Err(engram_core::Error::Validation(
            "sprint_id / mission_id 는 부모 epic 에서 자동 derive 되므로 직접 지정할 수 없습니다. 다른 sprint/mission 에 두려면 epic 을 옮기세요 (ADR-0014).".to_string()
        ));
    }
    let priority: Option<IssuePriority> = args["priority"].as_str()
        .and_then(|s| serde_json::from_value(serde_json::Value::String(s.to_string())).ok());
    let input = CreateIssueInput {
        epic_id:     args["epic_id"].as_i64().unwrap_or(0),
        title:       args["title"].as_str().unwrap_or("").to_string(),
        description: args["description"].as_str().map(String::from),
        goal:        args["goal"].as_str().map(String::from),
        priority,
    };
    Ok(serde_json::to_value(db.issue_create(input).await?).unwrap())
}

pub async fn get(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let id = args["id"].as_i64().unwrap_or(0);
    let compact = args["compact"].as_bool().unwrap_or(false);
    Ok(serde_json::to_value(db.issue_get(id, compact).await?).unwrap())
}

pub async fn list(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let mut status = None;
    let mut statuses = None;

    if let Some(s) = args["status"].as_str() {
        status = serde_json::from_value(serde_json::Value::String(s.to_string())).ok();
    } else if let Some(arr) = args["status"].as_array() {
        let mut list = Vec::new();
        for v in arr {
            if let Some(s) = v.as_str() {
                if let Ok(st) = serde_json::from_value::<IssueStatus>(serde_json::Value::String(s.to_string())) {
                    list.push(st);
                }
            }
        }
        statuses = Some(list);
    }

    let filter = IssueFilter {
        epic_id:      args["epic_id"].as_i64(),
        mission_id:   args["mission_id"].as_i64(),
        sprint_id:    args["sprint_id"].as_i64(),
        backlog_only: args["backlog_only"].as_bool().unwrap_or(false),
        project_key:  args["project_key"].as_str().map(String::from),
        status,
        statuses,
        priority:     None,
    };
    Ok(serde_json::to_value(db.issue_list(filter).await?).unwrap())
}

pub async fn stalled(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let threshold = args["threshold_minutes"].as_i64()
        .ok_or_else(|| engram_core::Error::Validation("threshold_minutes is required".to_string()))?;
    let status: IssueStatus = args["status"].as_str()
        .and_then(|s| serde_json::from_value(serde_json::Value::String(s.to_string())).ok())
        .unwrap_or(IssueStatus::Working);
    let project_key = args["project_key"].as_str();
    Ok(serde_json::to_value(db.stalled_issues(project_key, status, threshold).await?).unwrap())
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

    let agent_id = args["agent_id"].as_str()
        .ok_or_else(|| engram_core::Error::Validation("agent_id is required".to_string()))?;
    let input = UpdateIssueInput {
        status,
        priority,
        title,
        description,
        goal,
        epic_id: args["epic_id"].as_i64(),
    };
    Ok(serde_json::to_value(db.issue_update(id, input, agent_id).await?).unwrap())
}

pub async fn link(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let _agent_id = args["agent_id"].as_str()
        .ok_or_else(|| engram_core::Error::Validation("agent_id is required".to_string()))?;
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
    let _agent_id = args["agent_id"].as_str()
        .ok_or_else(|| engram_core::Error::Validation("agent_id is required".to_string()))?;
    let link_id = args["link_id"].as_i64().unwrap_or(0);
    db.issue_unlink(link_id).await?;
    Ok(serde_json::json!({ "ok": true, "deleted_id": link_id }))
}

pub async fn delete(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let id = args["id"].as_i64()
        .ok_or_else(|| engram_core::Error::Validation("id is required".to_string()))?;
    let agent_id = args["agent_id"].as_str().unwrap_or("agent");
    db.issue_delete(id, agent_id).await?;
    Ok(serde_json::json!({ "ok": true, "deleted_id": id }))
}

pub async fn claim(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let id = args["id"].as_i64()
        .ok_or_else(|| engram_core::Error::Validation("id is required".to_string()))?;
    let agent_id = args["agent_id"].as_str()
        .ok_or_else(|| engram_core::Error::Validation("agent_id is required for claim (멀티 에이전트 식별)".to_string()))?;
    Ok(serde_json::to_value(db.issue_claim(id, agent_id).await?).unwrap())
}

pub async fn release(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let id = args["id"].as_i64()
        .ok_or_else(|| engram_core::Error::Validation("id is required".to_string()))?;
    let agent_id = args["agent_id"].as_str()
        .ok_or_else(|| engram_core::Error::Validation("agent_id is required for release".to_string()))?;
    let transition_to: IssueStatus = args["transition_to"].as_str()
        .and_then(|s| serde_json::from_value(serde_json::Value::String(s.to_string())).ok())
        .ok_or_else(|| engram_core::Error::Validation("transition_to is required (ready|demo|required)".to_string()))?;
    let force = args["force"].as_bool().unwrap_or(false);
    Ok(serde_json::to_value(db.issue_release(id, transition_to, agent_id, force).await?).unwrap())
}

pub async fn planning_review_queue(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let project_key = args["project_key"].as_str()
        .ok_or_else(|| engram_core::Error::Validation("project_key is required".to_string()))?;
    let sprint_id = args["sprint_id"].as_i64();
    let statuses: Option<Vec<IssueStatus>> = if let Some(arr) = args["statuses"].as_array() {
        let mut result = Vec::new();
        for v in arr {
            let s = v.as_str().ok_or_else(|| engram_core::Error::Validation(
                format!("statuses 배열에 문자열이 아닌 값이 포함되어 있습니다: {:?}", v)
            ))?;
            let status = serde_json::from_value::<IssueStatus>(serde_json::Value::String(s.to_string()))
                .map_err(|_| engram_core::Error::Validation(
                    format!("invalid status: '{}'. 허용값: required, ready, working, demo, finished, cancelled", s)
                ))?;
            result.push(status);
        }
        Some(result)
    } else {
        None
    };
    let snapshot = db.planning_review_queue(project_key, sprint_id, statuses).await?;
    Ok(serde_json::to_value(&snapshot).unwrap())
}

pub async fn finish(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let id = args["id"].as_i64()
        .ok_or_else(|| engram_core::Error::Validation("id is required".to_string()))?;
    let agent_id = args["agent_id"].as_str()
        .ok_or_else(|| engram_core::Error::Validation("agent_id is required".to_string()))?;
    let m = db.issue_finish(id, agent_id).await?;
    Ok(serde_json::to_value(&m).unwrap())
}

pub async fn cancel(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let id = args["id"].as_i64()
        .ok_or_else(|| engram_core::Error::Validation("id is required".to_string()))?;
    let reason = args["reason"].as_str()
        .ok_or_else(|| engram_core::Error::Validation("reason is required".to_string()))?;
    let agent_id = args["agent_id"].as_str()
        .ok_or_else(|| engram_core::Error::Validation("agent_id is required".to_string()))?;
    let m = db.issue_cancel(id, reason, agent_id).await?;
    Ok(serde_json::to_value(&m).unwrap())
}

pub async fn bulk_update(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let ids: Vec<i64> = args["ids"].as_array()
        .ok_or_else(|| engram_core::Error::Validation("ids (integer array) is required".to_string()))?
        .iter()
        .map(|v| v.as_i64().unwrap_or(0))
        .collect();

    let status: Option<IssueStatus> = args["status"].as_str()
        .and_then(|s| serde_json::from_value(serde_json::Value::String(s.to_string())).ok());

    let priority: Option<IssuePriority> = args["priority"].as_str()
        .and_then(|s| serde_json::from_value(serde_json::Value::String(s.to_string())).ok());

    let agent_id = args["agent_id"].as_str()
        .ok_or_else(|| engram_core::Error::Validation("agent_id is required".to_string()))?;

    let input = BulkUpdateInput { status, priority };
    Ok(serde_json::to_value(db.issue_bulk_update(ids, input, agent_id).await?).unwrap())
}
