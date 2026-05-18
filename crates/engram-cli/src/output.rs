//! 모든 CLI 서브커맨드의 출력 직렬화를 단일 진입점으로 모은다.
//!
//! ADR-0010 에 따른 규약:
//! - 글로벌 `--json` 플래그 (또는 `--output {json|pretty|text}`) 가 지정되면 stdout 은
//!   머신 파서를 위해 **단일 JSON object/array 만** 출력. 이모지/배너 금지.
//! - 기본 모드는 `Pretty` — 사람이 읽기 좋은 들여쓴 JSON (기존 동작 보존).
//! - 에러는 항상 stderr 로. `Json` 모드에서는 `{"error":{"code":"...","message":"..."}}`.
//! - exit code 매핑은 `error_exit_code()` 참조.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// 머신 파싱용 — compact 한 줄 JSON + 후행 개행. 이모지/배너 금지.
    Json,
    /// 사람용 — 들여쓴 JSON. (현재 기본; 별도 텍스트 포맷이 정의되기 전까지는 Pretty 와 동일.)
    Pretty,
}

impl OutputFormat {
    pub fn from_flags(json: bool) -> Self {
        if json {
            OutputFormat::Json
        } else {
            OutputFormat::Pretty
        }
    }
}

/// 값(직렬화 가능) 을 stdout 에 출력. JSON 모드에선 compact, Pretty 모드에선 들여쓰기.
pub fn print_value<T: Serialize>(value: &T, fmt: OutputFormat) -> anyhow::Result<()> {
    let s = match fmt {
        OutputFormat::Json   => serde_json::to_string(value)?,
        OutputFormat::Pretty => serde_json::to_string_pretty(value)?,
    };
    println!("{s}");
    Ok(())
}

/// 사람용 메시지 — Pretty 모드에서만 stdout 으로 출력 (이모지 포함 가능).
/// Json 모드에서는 침묵 (머신 파서가 stdout 에서 단일 JSON 외 다른 텍스트를 만나지 않게).
pub fn print_human(msg: &str, fmt: OutputFormat) {
    if matches!(fmt, OutputFormat::Pretty) {
        println!("{msg}");
    }
}

/// 에러를 stderr 로 출력. Json 모드면 `{"error":{"code","message"}}` JSON,
/// Pretty 모드면 사람용 텍스트.
pub fn print_error(err: &anyhow::Error, fmt: OutputFormat) {
    let (code, message) = classify_error(err);
    match fmt {
        OutputFormat::Json => {
            let payload = serde_json::json!({
                "error": { "code": code, "message": message }
            });
            eprintln!("{}", payload);
        }
        OutputFormat::Pretty => {
            eprintln!("error[{code}]: {message}");
        }
    }
}

/// anyhow::Error 의 root cause 가 engram_core::Error 면 variant 별 code 를 반환.
/// 아니면 ("internal", msg).
pub fn classify_error(err: &anyhow::Error) -> (&'static str, String) {
    if let Some(e) = err.downcast_ref::<engram_core::Error>() {
        match e {
            engram_core::Error::Validation(m)        => ("validation", m.clone()),
            engram_core::Error::NotFound(m)          => ("not_found", m.clone()),
            engram_core::Error::InvalidTransition(m) => ("invalid_transition", m.clone()),
            engram_core::Error::Db(_) | engram_core::Error::Migration(_) => {
                ("internal", e.to_string())
            }
        }
    } else {
        ("internal", err.to_string())
    }
}

/// ADR-0010 의 exit code 매핑.
///   0 = ok (호출 측이 분기), 1 = 기타 (Db/Migration/그외 anyhow),
///   2 = Validation, 3 = NotFound, 4 = InvalidTransition (CAS 거부 포함).
///   clap 파싱 실패는 clap 기본(2) 사용.
pub fn error_exit_code(err: &anyhow::Error) -> i32 {
    if let Some(e) = err.downcast_ref::<engram_core::Error>() {
        match e {
            engram_core::Error::Validation(_)        => 2,
            engram_core::Error::NotFound(_)          => 3,
            engram_core::Error::InvalidTransition(_) => 4,
            engram_core::Error::Db(_) | engram_core::Error::Migration(_) => 1,
        }
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_validation() {
        let e: anyhow::Error = engram_core::Error::Validation("bad".into()).into();
        let (code, msg) = classify_error(&e);
        assert_eq!(code, "validation");
        assert_eq!(msg, "bad");
        assert_eq!(error_exit_code(&e), 2);
    }

    #[test]
    fn test_classify_not_found() {
        let e: anyhow::Error = engram_core::Error::NotFound("epic:99".into()).into();
        let (code, _) = classify_error(&e);
        assert_eq!(code, "not_found");
        assert_eq!(error_exit_code(&e), 3);
    }

    #[test]
    fn test_classify_invalid_transition() {
        let e: anyhow::Error = engram_core::Error::InvalidTransition("cas refused".into()).into();
        let (code, _) = classify_error(&e);
        assert_eq!(code, "invalid_transition");
        assert_eq!(error_exit_code(&e), 4);
    }

    #[test]
    fn test_classify_other_anyhow() {
        let e: anyhow::Error = anyhow::anyhow!("io thing");
        let (code, _) = classify_error(&e);
        assert_eq!(code, "internal");
        assert_eq!(error_exit_code(&e), 1);
    }

    #[test]
    fn test_print_value_json_is_compact() {
        // smoke — 직렬화 자체만 검증
        let v = serde_json::json!({ "a": 1, "b": "x" });
        let json   = serde_json::to_string(&v).unwrap();
        let pretty = serde_json::to_string_pretty(&v).unwrap();
        assert!(json.len() < pretty.len(), "Json 은 compact 여야 함");
        // valid JSON?
        let _: serde_json::Value = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn test_output_format_from_flags() {
        assert_eq!(OutputFormat::from_flags(true),  OutputFormat::Json);
        assert_eq!(OutputFormat::from_flags(false), OutputFormat::Pretty);
    }
}
