use std::time::{Duration, Instant};

pub const SUCCESS_TTL: Duration = Duration::from_secs(4);
pub const INFO_TTL: Duration = Duration::from_secs(4);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationKind {
    Success,
    Error,
    #[allow(dead_code)]
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub id: u64,
    pub kind: NotificationKind,
    pub message: String,
    pub created_at: Instant,
    /// None means the notification persists until manually dismissed.
    pub ttl: Option<Duration>,
}

impl Notification {
    pub fn success(id: u64, message: impl Into<String>) -> Self {
        Self {
            id,
            kind: NotificationKind::Success,
            message: message.into(),
            created_at: Instant::now(),
            ttl: Some(SUCCESS_TTL),
        }
    }

    pub fn error(id: u64, message: impl Into<String>) -> Self {
        Self {
            id,
            kind: NotificationKind::Error,
            message: message.into(),
            created_at: Instant::now(),
            ttl: None,
        }
    }

    pub fn info(id: u64, message: impl Into<String>) -> Self {
        Self {
            id,
            kind: NotificationKind::Info,
            message: message.into(),
            created_at: Instant::now(),
            ttl: Some(INFO_TTL),
        }
    }

    pub fn is_expired(&self) -> bool {
        match self.ttl {
            Some(ttl) => self.created_at.elapsed() >= ttl,
            None => false,
        }
    }
}
