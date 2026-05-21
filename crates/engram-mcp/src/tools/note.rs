use engram_core::{Db, models::note::*};
use serde_json::{json, Value};
use std::sync::Arc;

pub fn tool_definitions() -> Vec<Value> {
    vec![
        json!({ "name": "note_add",
            "description": "구조화된 노트를 추가합니다. note_type: caveat(주의)/decision(결정)/discovery(발견)/blocker_detail(블로커)/context(인수인계)/reference(참조)/comment(사용자-에이전트 대화). scope 로 적용 범위 선택: issue (기본, issue_id 필수), task (task_id 필수), project (project_key 필수), sprint/epic (scope_target_id 필수). broadcast scope (project/sprint/epic) 노트는 session_restore.active_caveats 에 자동 노출되어 어떤 이슈를 잡든 모든 에이전트가 본다.",
            "inputSchema": { "type": "object", "required": ["note_type", "summary"],
                "properties": {
                    "issue_id":  { "type": "integer", "description": "scope='issue'|'task' 일 때 필수." },
                    "task_id":   { "type": "integer" },
                    "note_type": { "type": "string", "enum": ["caveat","decision","discovery","blocker_detail","context","reference","comment"] },
                    "summary":   { "type": "string", "description": "한 줄 요약 (session_restore에서 항상 표시)" },
                    "detail":    { "type": "string", "description": "상세 내용 (길어도 됨, note_get으로만 로드)" },
                    "author":    { "type": "string", "description": "작성자 역할. 기본 'agent', 사용자 작성은 'user'." },
                    "agent_id":  { "type": "string", "description": "작성 에이전트 인스턴스 식별자 (예: 'claude-opus@sess-abc'). 옵셔널." },
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
                    "note_type": { "type": "string", "enum": ["caveat","decision","discovery","blocker_detail","context","reference","comment"] },
                    "include_resolved": { "type": "boolean" },
                    "include_detail": { "type": "boolean", "description": "detail 필드를 포함하여 조회할지 여부 (기본값 false)" }
                }
            }
        }),
        json!({ "name": "note_get", "description": "노트 상세를 조회합니다.",
            "inputSchema": { "type": "object", "required": ["note_id"],
                "properties": {
                    "note_id": { "type": "integer" },
                    "compact": { "type": "boolean", "description": "true인 경우 detail 필드를 NULL로 반환하여 대역폭 절약" }
                }
            }
        }),
        json!({ "name": "note_resolve", "description": "노트를 해결됨으로 표시합니다. 질문성 코멘트에 답변 후 원본 질문 노트를 종결할 때 사용하세요.",
            "inputSchema": { "type": "object", "required": ["note_id"],
                "properties": { "note_id": { "type": "integer" } }
            }
        }),
        json!({ "name": "note_add_bulk",
            "description": "구조화된 노트를 여러 개 한 번에 추가합니다. 트랜잭션 단위로 실행됩니다. 각 노트 입력 형식은 note_add 와 동일합니다.",
            "inputSchema": { "type": "object", "required": ["notes"],
                "properties": {
                    "notes": {
                        "type": "array",
                        "items": {
                            "type": "object", "required": ["note_type", "summary"],
                            "properties": {
                                "issue_id":  { "type": "integer" },
                                "task_id":   { "type": "integer" },
                                "note_type": { "type": "string", "enum": ["caveat","decision","discovery","blocker_detail","context","reference","comment"] },
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
    let input = CreateNoteInput {
        issue_id:  args["issue_id"].as_i64().unwrap_or(0),
        task_id:   args["task_id"].as_i64(),
        note_type,
        summary:   args["summary"].as_str().unwrap_or("").to_string(),
        detail:    args["detail"].as_str().map(String::from),
        author:    args["author"].as_str().map(String::from),
        agent_id:  args["agent_id"].as_str().map(String::from),
        scope,
        scope_target_id: args["scope_target_id"].as_i64(),
        project_key:     args["project_key"].as_str().map(String::from),
    };
    Ok(serde_json::to_value(db.note_add(input).await?).unwrap())
}

pub async fn list(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let issue_id = args["issue_id"].as_i64();
    let task_id  = args["task_id"].as_i64();
    let note_type = args["note_type"].as_str().and_then(parse_note_type);
    let include_resolved = args["include_resolved"].as_bool().unwrap_or(false);
    let include_detail = args["include_detail"].as_bool().unwrap_or(false);
    Ok(serde_json::to_value(db.note_list(issue_id, task_id, note_type, include_resolved, include_detail).await?).unwrap())
}

pub async fn get(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let id = args["note_id"].as_i64().unwrap_or(0);
    let compact = args["compact"].as_bool().unwrap_or(false);
    Ok(serde_json::to_value(db.note_get(id, compact).await?).unwrap())
}

pub async fn resolve(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let id = args["note_id"].as_i64().unwrap_or(0);
    let agent_id = args["agent_id"].as_str().unwrap_or("agent");
    Ok(serde_json::to_value(db.note_resolve(id, agent_id).await?).unwrap())
}

pub async fn add_bulk(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let list = args["notes"].as_array()
        .ok_or_else(|| engram_core::Error::Validation("notes array is required".to_string()))?;
    let mut inputs = Vec::new();
    for v in list {
        let note_type = v["note_type"].as_str()
            .and_then(parse_note_type)
            .ok_or_else(|| engram_core::Error::Validation(
                format!("invalid note_type: {:?}. 허용값: caveat, decision, discovery, blocker_detail, context, reference, comment", v["note_type"])
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
            agent_id:  v["agent_id"].as_str().map(String::from),
            scope,
            scope_target_id: v["scope_target_id"].as_i64(),
            project_key:     v["project_key"].as_str().map(String::from),
        });
    }
    Ok(serde_json::to_value(db.note_add_bulk(inputs).await?).unwrap())
}
