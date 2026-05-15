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
            "tools/list" => self.handle_tools_list(id),
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

    fn handle_tools_list(&self, id: Value) -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": { "tools": crate::tools::all_tool_definitions() }
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
