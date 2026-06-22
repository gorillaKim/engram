use engram_core::{Db, models::note::*};
use serde_json::{json, Value};
use std::sync::Arc;

pub fn tool_definitions() -> Vec<Value> {
    vec![
        json!({ "name": "note_add",
            "description": r#"note_add — 이슈/태스크/스프린트/에픽/프로젝트 범위 노트 추가.

scope 별 필수 필드:
- scope="issue"   → issue_id 필수
- scope="task"    → task_id 필수
- scope="sprint"  → scope_target_id (sprint_id) + scope="sprint" 필수
- scope="epic"    → scope_target_id (epic_id) + scope="epic" 필수
- scope="project" → project_key 필수

note_type: caveat | decision | discovery | blocker_detail | context | reference | comment
- caveat: 함정/주의 (broadcast 가능, session_restore 자동 노출)
- decision: 의사결정 기록
- discovery: 작업 중 발견
- context: 인수인계 (demo 진입 전 필수)
- blocker_detail: 블로커 상세"#,
            "inputSchema": { "type": "object", "required": ["note_type", "summary", "agent_id"],
                "properties": {
                    "issue_id":  { "type": "integer", "description": "scope='issue'|'task' 일 때 필수." },
                    "task_id":   { "type": "integer" },
                    "note_type": { "type": "string", "enum": ["caveat","decision","discovery","blocker_detail","context","reference","comment","evaluation"] },
                    "summary":   { "type": "string", "description": "한 줄 요약 (session_restore에서 항상 표시)" },
                    "detail":    { "type": "string", "description": "상세 내용 (길어도 됨, note_get으로만 로드)" },
                    "omit_detail": { "type": "boolean", "description": "true인 경우 반환값에서 detail 필드를 생략하여 대역폭 절약" },
                    "author":    { "type": "string", "description": "작성자 역할. 기본 'agent', 사용자 작성은 'user'." },
                    "agent_id":  { "type": "string", "description": "작성 에이전트 인스턴스 식별자 (예: 'claude-opus@sess-abc')." },
                    "scope":     { "type": "string", "enum": ["project","sprint","epic","issue","task"], "description": "노트 적용 범위. 생략 시 issue 또는 task 자동 판정." },
                    "scope_target_id": { "type": "integer", "description": "scope='sprint'|'epic' 일 때 해당 entity id." },
                    "project_key": { "type": "string", "description": "scope='project' 일 때 필수." }
                }
            }
        }),
        json!({ "name": "note_list",
            "description": "노트 목록을 조회합니다 (기본적으로 detail은 제외하는 compact 모드). note_type 으로 필터 가능. 코멘트만 볼 때 note_type='comment'.",
            "inputSchema": { "type": "object",
                "properties": {
                    "issue_id": { "type": "integer" },
                    "task_id":  { "type": "integer" },
                    "note_type": { "type": "string", "enum": ["caveat","decision","discovery","blocker_detail","context","reference","comment","evaluation"] },
                    "note_types": {
                        "type": "array",
                        "items": { "type": "string", "enum": ["caveat","decision","discovery","blocker_detail","context","reference","comment","evaluation"] }
                    },
                    "include_resolved": { "type": "boolean" },
                    "include_detail": { "type": "boolean", "description": "detail 필드를 포함하여 조회할지 여부 (기본값 false)" },
                    "project_key": { "type": "string", "description": "특정 프로젝트의 노트만 필터" },
                    "sprint_id": { "type": "integer", "description": "특정 스프린트의 노트만 필터" },
                    "epic_id": { "type": "integer", "description": "특정 에픽의 노트만 필터" },
                    "rollup": { "type": "boolean", "description": "true일 경우 에픽 하위 이슈 및 태스크의 노트를 포함하여 조회 (기본값 false)" },
                    "limit":            { "type": "integer" },
                    "offset":           { "type": "integer" },
                    "compact":          { "type": "boolean" },
                    "projection":       { "type": "array", "items": { "type": "string" } },
                    "updated_after": { "type": "string", "description": "이 시각 이후에 업데이트/생성된 노트만 필터 (YYYY-MM-DD HH:MM:SS)" },
                    "mode": {
                        "type": "string",
                        "enum": ["normal", "compact", "agent"],
                        "description": "출력 모드. 기본값은 'agent' (영문 요약 텍스트). 'compact' 또는 'normal' 선택 가능"
                    }
                }
            }
        }),
        json!({ "name": "note_get", "description": "노트 상세를 일괄 또는 단건 조회합니다. id 에 단일 정수 또는 정수 배열을 넘길 수 있습니다.",
            "inputSchema": { "type": "object", "required": ["id"],
                "properties": {
                    "id": {
                        "oneOf": [
                            { "type": "integer" },
                            { "type": "array", "items": { "type": "integer" } }
                        ]
                    },
                    "compact": { "type": "boolean", "description": "true인 경우 detail 필드를 NULL로 반환하여 대역폭 절약" }
                }
            }
        }),
        json!({ "name": "note_resolve", "description": "노트를 해결됨으로 표시합니다. 질문성 코멘트에 답변 후 원본 질문 노트를 종결할 때 사용하세요.",
            "inputSchema": { "type": "object", "required": ["id", "agent_id"],
                "properties": {
                    "id":       { "type": "integer" },
                    "agent_id": { "type": "string", "description": "호출 액터 식별자" }
                }
            }
        }),
        json!({ "name": "note_add_bulk",
            "description": "구조화된 노트를 여러 개 한 번에 추가합니다. 트랜잭션 단위로 실행됩니다. 각 노트 입력 형식은 note_add 와 동일합니다.",
            "inputSchema": { "type": "object", "required": ["notes", "agent_id"],
                "properties": {
                    "agent_id": { "type": "string", "description": "호출 액터 식별자" },
                    "omit_detail": { "type": "boolean", "description": "true인 경우 반환값들에서 detail 필드를 생략하여 대역폭 절약" },
                    "notes": {
                        "type": "array",
                        "items": {
                            "type": "object", "required": ["note_type", "summary"],
                            "properties": {
                                "issue_id":  { "type": "integer" },
                                "task_id":   { "type": "integer" },
                                "note_type": { "type": "string", "enum": ["caveat","decision","discovery","blocker_detail","context","reference","comment","evaluation"] },
                                "summary":   { "type": "string" },
                                "detail":    { "type": "string" },
                                "author":    { "type": "string" },
                                "agent_id":  { "type": "string" },
                                "scope":     { "type": "string", "enum": ["project","sprint","epic","issue","task"] },
                                "scope_target_id": { "type": "integer" },
                                "project_key": { "type": "string" }
                            }
                        }
                    }
                }
            }
        }),
    ]
}

fn parse_note_type(s: &str) -> Option<NoteType> {
    match s {
        "caveat"         => Some(NoteType::Caveat),
        "decision"       => Some(NoteType::Decision),
        "discovery"      => Some(NoteType::Discovery),
        "blocker_detail" => Some(NoteType::BlockerDetail),
        "context"        => Some(NoteType::Context),
        "reference"      => Some(NoteType::Reference),
        "comment"        => Some(NoteType::Comment),
        "evaluation"     => Some(NoteType::Evaluation),
        _ => None,
    }
}

pub async fn add(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let note_type = args["note_type"].as_str()
        .and_then(parse_note_type)
        .unwrap_or(NoteType::Context);
    let scope = args["scope"].as_str().and_then(|s| match s {
        "project" => Some(engram_core::models::note::NoteScope::Project),
        "sprint"  => Some(engram_core::models::note::NoteScope::Sprint),
        "epic"    => Some(engram_core::models::note::NoteScope::Epic),
        "issue"   => Some(engram_core::models::note::NoteScope::Issue),
        "task"    => Some(engram_core::models::note::NoteScope::Task),
        _ => None,
    });
    let agent_id = args["agent_id"].as_str()
        .ok_or_else(|| engram_core::Error::Validation("agent_id is required".to_string()))?;
    let input = CreateNoteInput {
        issue_id:  args["issue_id"].as_i64().unwrap_or(0),
        task_id:   args["task_id"].as_i64(),
        note_type,
        summary:   args["summary"].as_str().unwrap_or("").to_string(),
        detail:    args["detail"].as_str().map(String::from),
        author:    args["author"].as_str().map(String::from),
        agent_id:  Some(agent_id.to_string()),
        scope,
        scope_target_id: args["scope_target_id"].as_i64(),
        project_key:     args["project_key"].as_str().map(String::from),
    };
    let note = db.note_add(input).await?;
    let mode = super::get_mode(args);
    if matches!(mode, engram_core::models::OutputMode::Agent) {
        Ok(super::format::success(&format!(
            "Note #{} created.",
            note.id
        )))
    } else {
        Ok(json!({ "id": note.id, "status": "ok" }))
    }
}

pub async fn list(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let issue_id = args["issue_id"].as_i64();
    let task_id  = args["task_id"].as_i64();
    let note_type = args["note_type"].as_str().and_then(parse_note_type);
    let mut note_types = None;
    if let Some(arr) = args["note_types"].as_array() {
        let mut list = Vec::new();
        for v in arr {
            if let Some(s) = v.as_str() {
                if let Some(nt) = parse_note_type(s) {
                    list.push(nt);
                }
            }
        }
        note_types = Some(list);
    }
    let include_resolved = args["include_resolved"].as_bool().unwrap_or(false);
    let include_detail = args["include_detail"].as_bool().unwrap_or(false);
    let project_key = args["project_key"].as_str();
    let sprint_id = args["sprint_id"].as_i64();
    let epic_id = args["epic_id"].as_i64();
    let rollup = args["rollup"].as_bool();
    let limit = args["limit"].as_i64();
    let offset = args["offset"].as_i64();
    let mode = super::get_mode(args);
    let updated_after = args["updated_after"].as_str().map(String::from);

    let response = db.note_list_mode(
        issue_id,
        task_id,
        note_type,
        note_types,
        include_resolved,
        include_detail,
        project_key,
        sprint_id,
        epic_id,
        rollup,
        limit,
        offset,
        mode,
        updated_after,
    ).await?;

    let mut val = match response {
        engram_core::models::CoreResponse::Text(s) => return Ok(Value::String(s)),
        engram_core::models::CoreResponse::Json(j) => serde_json::to_value(&j).unwrap(),
    };
    if let Some(arr) = args["projection"].as_array() {
        let fields: Vec<String> = arr.iter().filter_map(|v| v.as_str().map(String::from)).collect();
        val = engram_core::apply_projection(val, &fields);
    }
    Ok(val)
}

pub async fn get(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let compact = args["compact"].as_bool().unwrap_or(false);
    let mode = super::get_mode(args);

    if let Some(arr) = args["id"].as_array() {
        let ids: Vec<i64> = arr.iter().filter_map(|v| v.as_i64()).collect();
        let response = db.note_get_batch(&ids, compact, mode).await?;
        match response {
            engram_core::models::CoreResponse::Text(s) => Ok(Value::String(s)),
            engram_core::models::CoreResponse::Json(j) => Ok(serde_json::to_value(j).unwrap()),
        }
    } else {
        let id = args["id"].as_i64().ok_or_else(|| engram_core::Error::Validation("id required".into()))?;
        let note = db.note_get(id, compact).await?;
        if matches!(mode, engram_core::models::OutputMode::Agent) {
            let note_type_val = serde_json::to_value(&note.note_type).unwrap();
            let note_type_str = note_type_val.as_str().unwrap_or("context");
            let resolved_str = if note.resolved { "Yes" } else { "No" };
            let mut fields = vec![
                ("ID", note.id.to_string()),
                ("Issue ID", note.issue_id.map(|id| id.to_string()).unwrap_or_else(|| "-".to_string())),
                ("Type", note_type_str.to_string()),
                ("Summary", note.summary.clone()),
                ("Resolved", resolved_str.to_string()),
            ];
            if let Some(ref detail) = note.detail {
                fields.push(("Detail", detail.clone()));
            }
            Ok(Value::String(super::format::make_details("Note Specification", &fields)))
        } else {
            Ok(serde_json::to_value(note).unwrap())
        }
    }
}

pub async fn resolve(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let id = args["id"].as_i64().ok_or_else(|| engram_core::Error::Validation("id required".into()))?;
    let agent_id = args["agent_id"].as_str()
        .ok_or_else(|| engram_core::Error::Validation("agent_id is required".to_string()))?;
    db.note_resolve(id, agent_id).await?;
    let mode = super::get_mode(args);
    if matches!(mode, engram_core::models::OutputMode::Agent) {
        Ok(super::format::success(&format!(
            "Note #{} resolved.",
            id
        )))
    } else {
        Ok(json!({ "id": id, "resolved": true, "status": "ok" }))
    }
}

pub async fn add_bulk(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let agent_id = args["agent_id"].as_str()
        .ok_or_else(|| engram_core::Error::Validation("agent_id is required".to_string()))?;
    let list = args["notes"].as_array()
        .ok_or_else(|| engram_core::Error::Validation("notes array is required".to_string()))?;
    let mut inputs = Vec::new();
    for v in list {
        let note_type = v["note_type"].as_str()
            .and_then(parse_note_type)
            .ok_or_else(|| engram_core::Error::Validation(
                format!("invalid note_type: {:?}. 허용값: caveat, decision, discovery, blocker_detail, context, reference, comment, evaluation", v["note_type"])
            ))?;
        let scope = v["scope"].as_str().and_then(|s| match s {
            "project" => Some(engram_core::models::note::NoteScope::Project),
            "sprint"  => Some(engram_core::models::note::NoteScope::Sprint),
            "epic"    => Some(engram_core::models::note::NoteScope::Epic),
            "issue"   => Some(engram_core::models::note::NoteScope::Issue),
            "task"    => Some(engram_core::models::note::NoteScope::Task),
            _ => None,
        });
        inputs.push(CreateNoteInput {
            issue_id:  v["issue_id"].as_i64().unwrap_or(0),
            task_id:   v["task_id"].as_i64(),
            note_type,
            summary:   v["summary"].as_str().unwrap_or("").to_string(),
            detail:    v["detail"].as_str().map(String::from),
            author:    v["author"].as_str().map(String::from),
            agent_id:  Some(v["agent_id"].as_str().unwrap_or(agent_id).to_string()),
            scope,
            scope_target_id: v["scope_target_id"].as_i64(),
            project_key:     v["project_key"].as_str().map(String::from),
        });
    }
    let notes = db.note_add_bulk(inputs).await?;
    let mode = super::get_mode(args);
    if matches!(mode, engram_core::models::OutputMode::Agent) {
        Ok(super::format::success(&format!(
            "Notes created (Count: {}).",
            notes.len()
        )))
    } else {
        let ids: Vec<Value> = notes.iter().map(|n| json!({ "id": n.id, "status": "ok" })).collect();
        Ok(json!(ids))
    }
}
