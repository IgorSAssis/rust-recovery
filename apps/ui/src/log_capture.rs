use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use tracing::Level;
use tracing_subscriber::Layer;

pub type LogBuffer = Arc<Mutex<VecDeque<LogEntry>>>;

const MAX_ENTRIES: usize = 500;
const ALLOWED_TARGETS: &[&str] = &["recovery_engine", "file_carver", "device_detector", "ui"];

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub level: Level,
    pub target: String,
    pub message: String,
    pub timestamp: SystemTime,
}

pub struct LogCaptureLayer {
    buffer: LogBuffer,
}

impl LogCaptureLayer {
    pub fn new(buffer: LogBuffer) -> Self {
        Self { buffer }
    }
}

impl<S: tracing::Subscriber> Layer<S> for LogCaptureLayer {
    fn enabled(
        &self,
        metadata: &tracing::Metadata<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) -> bool {
        let target = metadata.target();
        *metadata.level() <= Level::INFO && ALLOWED_TARGETS.iter().any(|&t| target.starts_with(t))
    }

    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let meta = event.metadata();
        let mut visitor = MessageVisitor(String::new());
        event.record(&mut visitor);

        let entry = LogEntry {
            level: *meta.level(),
            target: meta.target().to_string(),
            message: visitor.0,
            timestamp: SystemTime::now(),
        };

        if let Ok(mut buf) = self.buffer.lock() {
            if buf.len() >= MAX_ENTRIES {
                buf.pop_front();
            }
            buf.push_back(entry);
        }
    }
}

struct MessageVisitor(String);

impl tracing::field::Visit for MessageVisitor {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.0 = value.to_string();
        }
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.0 = format!("{:?}", value);
        }
    }
}
