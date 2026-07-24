use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopSettings {
    pub mcp: McpSettings,
    #[serde(default)]
    pub activity: ActivitySettings,
    #[serde(default)]
    pub prompt: PromptSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpSettings {
    pub autostart: bool,
    pub port: u16,
}

/// working 이슈 활동 상태 분류 임계값 (분 단위)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivitySettings {
    /// 이 시간 초과 시 "작업예상" (경고) 상태로 전환. 기본 30분.
    pub warn_minutes: i64,
    /// 이 시간 초과 시 "작업중단" (에러) 상태로 전환. 기본 120분.
    pub stall_minutes: i64,
}

impl Default for ActivitySettings {
    fn default() -> Self {
        Self { warn_minutes: 30, stall_minutes: 120 }
    }
}

fn default_template() -> String {
    "{{base prompt}}".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptSettings {
    #[serde(default = "default_template")]
    pub issue_template: String,
    #[serde(default = "default_template")]
    pub epic_template: String,
    #[serde(default = "default_template")]
    pub mission_template: String,
    #[serde(default = "default_template")]
    pub retrospective_template: String,
}

impl Default for PromptSettings {
    fn default() -> Self {
        Self {
            issue_template: default_template(),
            epic_template: default_template(),
            mission_template: default_template(),
            retrospective_template: default_template(),
        }
    }
}

impl Default for DesktopSettings {
    fn default() -> Self {
        Self {
            mcp: McpSettings { autostart: true, port: 3456 },
            activity: ActivitySettings::default(),
            prompt: PromptSettings::default(),
        }
    }
}

fn settings_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".engram")
        .join("desktop.toml")
}

pub fn load() -> anyhow::Result<DesktopSettings> {
    let path = settings_path();
    if !path.exists() {
        return Ok(DesktopSettings::default());
    }
    let content = std::fs::read_to_string(&path)?;
    let settings: DesktopSettings = toml::from_str(&content)?;
    Ok(settings)
}

pub fn save(s: &DesktopSettings) -> anyhow::Result<()> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(s)?;
    // Atomic write via temp file
    let tmp = path.with_extension("toml.tmp");
    std::fs::write(&tmp, content)?;
    std::fs::rename(&tmp, &path)?;
    Ok(())
}

pub fn set_autostart(on: bool) -> anyhow::Result<()> {
    let mut s = load()?;
    s.mcp.autostart = on;
    save(&s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_default_values() {
        let s = DesktopSettings::default();
        assert_eq!(s.mcp.port, 3456);
        assert!(s.mcp.autostart);
        assert_eq!(s.prompt.issue_template, "{{base prompt}}");
        assert_eq!(s.prompt.epic_template, "{{base prompt}}");
        assert_eq!(s.prompt.mission_template, "{{base prompt}}");
        assert_eq!(s.prompt.retrospective_template, "{{base prompt}}");
    }

    #[test]
    fn test_settings_toml_roundtrip() {
        let s = DesktopSettings {
            mcp: McpSettings { autostart: false, port: 4000 },
            activity: Default::default(),
            prompt: PromptSettings {
                issue_template: "{{base prompt}}\n[issue]".into(),
                epic_template: "{{base prompt}}\n[epic]".into(),
                mission_template: "{{base prompt}}\n[mission]".into(),
                retrospective_template: "{{base prompt}}\n[retrospective]".into(),
            },
        };
        let toml_str = toml::to_string_pretty(&s).unwrap();
        let loaded: DesktopSettings = toml::from_str(&toml_str).unwrap();
        assert_eq!(loaded.mcp.port, 4000);
        assert!(!loaded.mcp.autostart);
        assert_eq!(loaded.prompt.issue_template, "{{base prompt}}\n[issue]");
        assert_eq!(loaded.prompt.epic_template, "{{base prompt}}\n[epic]");
        assert_eq!(loaded.prompt.mission_template, "{{base prompt}}\n[mission]");
        assert_eq!(loaded.prompt.retrospective_template, "{{base prompt}}\n[retrospective]");
    }
}
