use engram_core::{Db, models::note::*};
use serde_json::{json, Value};
use std::sync::Arc;

pub fn tool_definitions() -> Vec<Value> {
    vec![
        json!({ "name": "note_add",
            "description": "이슈/태스크에 구조화된 노트를 추가합니다. caveat(주의사항), decision(결정), discovery(발견), blocker_detail(블로커), context(인수인계), reference(참조) 중 선택하세요.",
            "inputSchema": { "type": "object", "required": ["issue_id", "note_type", "summary"],
                "properties": {
                    "issue_id":  { "type": "integer" },
                    "task_id":   { "type": "integer" },
                    "note_type": { "type": "string", "enum": ["caveat","decision","discovery","blocker_detail","context","reference"] },
                    "summary":   { "type": "string", "description": "한 줄 요약 (session_restore에서 항상 표시)" },
                    "detail":    { "type": "string", "description": "상세 내용 (길어도 됨, note_get으로만 로드)" }
                }
            }
        }),
        json!({ "name": "note_list", "description": "노트 목록을 조회합니다 (summary만 반환).",
            "inputSchema": { "type": "object",
                "properties": { "issue_id": { "type": "integer" }, "task_id": { "type": "integer" }, "type_filter": { "type": "string" }, "include_resolved": { "type": "boolean" } }
            }
        }),
        json!({ "name": "note_get", "description": "노트 상세를 조회합니다 (detail 포함).",
            "inputSchema": { "type": "object", "required": ["note_id"],
                "properties": { "note_id": { "type": "integer" } }
            }
        }),
        json!({ "name": "note_resolve", "description": "노트를 해결됨으로 표시합니다.",
            "inputSchema": { "type": "object", "required": ["note_id"],
                "properties": { "note_id": { "type": "integer" } }
            }
        }),
    ]
}

pub async fn add(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let note_type = match args["note_type"].as_str().unwrap_or("context") {
        "caveat"        => NoteType::Caveat,
        "decision"      => NoteType::Decision,
        "discovery"     => NoteType::Discovery,
        "blocker_detail"=> NoteType::BlockerDetail,
        "reference"     => NoteType::Reference,
        _               => NoteType::Context,
    };
    let input = CreateNoteInput {
        issue_id:  args["issue_id"].as_i64().unwrap_or(0),
        task_id:   args["task_id"].as_i64(),
        note_type,
        summary:   args["summary"].as_str().unwrap_or("").to_string(),
        detail:    args["detail"].as_str().map(String::from),
        author:    args["author"].as_str().map(String::from),
    };
    Ok(serde_json::to_value(db.note_add(input).await?).unwrap())
}

pub async fn list(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let issue_id = args["issue_id"].as_i64();
    let task_id  = args["task_id"].as_i64();
    let include_resolved = args["include_resolved"].as_bool().unwrap_or(false);
    Ok(serde_json::to_value(db.note_list(issue_id, task_id, None, include_resolved).await?).unwrap())
}

pub async fn get(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let id = args["note_id"].as_i64().unwrap_or(0);
    Ok(serde_json::to_value(db.note_get(id).await?).unwrap())
}

pub async fn resolve(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let id = args["note_id"].as_i64().unwrap_or(0);
    Ok(serde_json::to_value(db.note_resolve(id, "agent").await?).unwrap())
}
