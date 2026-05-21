use engram_core::Db;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

pub struct EngramMcpServer {
    db: Arc<Db>,
}

impl EngramMcpServer {
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }

    pub async fn run_stdio(self) -> anyhow::Result<()> {
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        loop {
            line.clear();
            let n = reader.read_line(&mut line).await?;
            if n == 0 { break; } // EOF

            let request: Value = match serde_json::from_str(line.trim()) {
                Ok(v) => v,
                Err(e) => {
                    tracing::warn!("Invalid JSON-RPC: {e}");
                    continue;
                }
            };

            let response = self.handle(&request).await;
            let mut out = serde_json::to_string(&response)?;
            out.push('\n');
            stdout.write_all(out.as_bytes()).await?;
            stdout.flush().await?;
        }
        Ok(())
    }

    pub async fn handle_request(&self, req: &Value) -> Value {
        self.handle(req).await
    }

    async fn handle(&self, req: &Value) -> Value {
        let id = req.get("id").cloned().unwrap_or(Value::Null);
        let method = req["method"].as_str().unwrap_or("");

        match method {
            "initialize" => self.handle_initialize(id),
            "tools/list" => self.handle_tools_list(req, id),
            "tools/call" => self.handle_tools_call(req, id).await,
            _ => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32601, "message": "Method not found" }
            }),
        }
    }

    fn handle_initialize(&self, id: Value) -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "engram", "version": env!("CARGO_PKG_VERSION") }
            }
        })
    }

    fn handle_tools_list(&self, req: &Value, id: Value) -> Value {
        let compact = req.get("params")
            .and_then(|p| p.get("compact"))
            .and_then(|c| c.as_bool())
            .unwrap_or(false);

        let mut tools = crate::tools::all_tool_definitions();
        if compact {
            for t in &mut tools {
                if let Some(desc) = t["description"].as_str() {
                    let short_desc = desc.split('.').next().unwrap_or(desc).to_string();
                    t["description"] = json!(short_desc);
                }
                if let Some(schema) = t["inputSchema"].as_object_mut() {
                    schema.remove("properties");
                    schema.remove("required");
                }
            }
        }

        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": { "tools": tools }
        })
    }

    async fn handle_tools_call(&self, req: &Value, id: Value) -> Value {
        let params = &req["params"];
        let tool_name = params["name"].as_str().unwrap_or("");
        let args = &params["arguments"];

        let result = crate::tools::dispatch(Arc::clone(&self.db), tool_name, args).await;

        match result {
            Ok(content) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": [{ "type": "text", "text": serde_json::to_string_pretty(&content).unwrap() }]
                }
            }),
            Err(e) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32000, "message": e.to_string() }
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engram_core::Db;

    #[tokio::test]
    async fn test_handle_tools_list_compact() {
        let db = Db::open_in_memory().await.unwrap();
        let server = EngramMcpServer::new(Arc::new(db));

        // 1) 기본 tools/list 호출
        let req_normal = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list"
        });
        let res_normal = server.handle(&req_normal).await;
        let tools_normal = res_normal["result"]["tools"].as_array().unwrap();
        // 적어도 하나 이상의 도구에 properties 가 있는지 검증
        let has_properties = tools_normal.iter().any(|t| {
            t["inputSchema"].get("properties").is_some()
        });
        assert!(has_properties, "Normal tools/list must include properties");

        // 2) compact tools/list 호출
        let req_compact = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {
                "compact": true
            }
        });
        let res_compact = server.handle(&req_compact).await;
        let tools_compact = res_compact["result"]["tools"].as_array().unwrap();
        // 모든 도구의 properties 와 required 가 없어야 함
        for t in tools_compact {
            assert!(t["inputSchema"].get("properties").is_none());
            assert!(t["inputSchema"].get("required").is_none());
        }
    }
}
