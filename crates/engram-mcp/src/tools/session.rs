use engram_core::Db;
use serde_json::{json, Value};
use std::sync::Arc;

pub fn tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "session_restore",
            "description": "세션 시작 시 반드시 호출하세요. 현재 활성 스프린트의 에픽/이슈 진행 현황, 미완료 태스크, 주의사항(caveat) 목록, 다음 처리할 태스크를 반환합니다. project_key를 지정하면 해당 프로젝트의 컨텍스트만 반환합니다.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_key": {
                        "type": "string",
                        "description": "필터할 프로젝트 식별자 (예: 'xpert-da-web'). 미입력 시 전체 반환"
                    },
                    "compact": {
                        "type": "boolean",
                        "description": "true면 노트/태스크를 count만 반환 (페이로드 70% 감소)"
                    },
                    "size_limit": {
                        "type": "integer",
                        "description": "응답 크기 한도 (기본 25000자)"
                    },
                    "mode": {
                        "type": "string",
                        "enum": ["normal", "compact", "agent"],
                        "description": "출력 모드. 기본값은 'agent' (영문 요약 텍스트). 'compact' 또는 'normal' 선택 가능"
                    }
                }
            }
        }),
        json!({
            "name": "session_end",
            "description": "세션 종료 전 반드시 호출하세요. context note 누락 여부와 미완료 in_progress 태스크를 확인합니다. warnings가 비어있으면 정상 종료입니다.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_key": {
                        "type": "string",
                        "description": "확인할 프로젝트 식별자. 미입력 시 전체 확인"
                    }
                }
            }
        }),
        json!({
            "name": "board_status",
            "description": "현재 스프린트의 전체 칸반 보드 현황을 반환합니다. 프로젝트별 에픽/이슈 분포와 블로킹 체인을 포함합니다.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_key": {
                        "type": "string",
                        "description": "특정 프로젝트만 조회. 미입력 시 전체"
                    },
                    "compact": {
                        "type": "boolean",
                        "description": "true 시 blocked_chains를 { blocker_id: [blocked_id, ...] } 형태로 압축"
                    },
                    "include_chains": {
                        "type": "boolean",
                        "description": "false 시 blocked_chains 필드를 응답에서 제외 (기본값 true)"
                    },
                    "mode": {
                        "type": "string",
                        "enum": ["normal", "compact", "agent"],
                        "description": "출력 모드. 기본값은 'agent' (영문 요약 텍스트). 'compact' 또는 'normal' 선택 가능"
                    }
                }
            }
        }),
    ]
}

pub async fn restore(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let project_key = args["project_key"].as_str();
    let stall_minutes = args["stall_minutes"].as_i64().unwrap_or(120);
    let size_limit = args["size_limit"].as_u64().map(|n| n as usize);

    let mode = if let Some(m_str) = args["mode"].as_str() {
        match m_str {
            "normal" => engram_core::models::OutputMode::Normal,
            "compact" => engram_core::models::OutputMode::Compact,
            "agent" => engram_core::models::OutputMode::Agent,
            _ => engram_core::models::OutputMode::Agent,
        }
    } else {
        if let Some(compact) = args["compact"].as_bool() {
            if compact {
                engram_core::models::OutputMode::Compact
            } else {
                engram_core::models::OutputMode::Normal
            }
        } else {
            engram_core::models::OutputMode::Agent
        }
    };

    let response = db.session_restore_mode(project_key, mode, stall_minutes, size_limit).await?;
    Ok(serde_json::to_value(response).unwrap())
}

pub async fn end(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let project_key = args["project_key"].as_str();
    let result = db.session_end(project_key).await?;
    Ok(serde_json::to_value(result).unwrap())
}

pub async fn board_status(db: Arc<Db>, args: &Value) -> engram_core::Result<Value> {
    let project_key = args["project_key"].as_str();
    
    let mode = if let Some(m_str) = args["mode"].as_str() {
        match m_str {
            "normal" => engram_core::models::OutputMode::Normal,
            "compact" => engram_core::models::OutputMode::Compact,
            "agent" => engram_core::models::OutputMode::Agent,
            _ => engram_core::models::OutputMode::Agent,
        }
    } else {
        if let Some(compact) = args["compact"].as_bool() {
            if compact {
                engram_core::models::OutputMode::Compact
            } else {
                engram_core::models::OutputMode::Normal
            }
        } else {
            engram_core::models::OutputMode::Agent
        }
    };

    let include_chains = args["include_chains"].as_bool().unwrap_or(true);
    let board = db.board_status_mode(project_key, mode, include_chains).await?;
    Ok(serde_json::to_value(board).unwrap())
}
