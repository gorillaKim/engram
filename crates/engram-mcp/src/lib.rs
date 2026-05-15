// engram-mcp library: imported by engram-desktop as an embedded MCP host.
// tracing_subscriber::init() must NOT be called here — only in entry-point binaries.
pub mod http;
pub mod server;
pub mod tools;
