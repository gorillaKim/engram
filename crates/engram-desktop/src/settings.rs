use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopSettings {
    pub mcp: McpSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpSettings {
    pub autostart: bool,
    pub port: u16,
}

impl Default for DesktopSettings {
    fn default() -> Self {
        Self {
            mcp: McpSettings { autostart: true, port: 3456 },
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
    }

    #[test]
    fn test_settings_toml_roundtrip() {
        let s = DesktopSettings {
            mcp: McpSettings { autostart: false, port: 4000 },
        };
        let toml_str = toml::to_string_pretty(&s).unwrap();
        let loaded: DesktopSettings = toml::from_str(&toml_str).unwrap();
        assert_eq!(loaded.mcp.port, 4000);
        assert!(!loaded.mcp.autostart);
    }
}
