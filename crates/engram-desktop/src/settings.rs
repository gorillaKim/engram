// M1 stub — MCP autostart and port config filled in M3
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppSettings {
    pub mcp_port: u16,
    pub mcp_autostart: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self { mcp_port: 3456, mcp_autostart: true }
    }
}
