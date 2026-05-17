use crate::mcp_supervisor::LogLine;
use chrono::Utc;
use tokio::sync::broadcast;
use tracing::{Event, Subscriber};
use tracing_subscriber::{layer::Context, Layer};

pub struct BroadcastLayer {
    pub tx: broadcast::Sender<LogLine>,
}

impl<S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>> Layer<S>
    for BroadcastLayer
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let meta = event.metadata();
        let level = meta.level().to_string();
        let target = meta.target().to_string();

        // Extract message via visitor
        let mut visitor = MsgVisitor(String::new());
        event.record(&mut visitor);

        let line = LogLine {
            level,
            target,
            msg: visitor.0,
            ts: Utc::now(),
        };
        let _ = self.tx.send(line);
    }
}

struct MsgVisitor(String);

impl tracing::field::Visit for MsgVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.0 = format!("{value:?}").trim_matches('"').to_string();
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.0 = value.to_string();
        }
    }
}
