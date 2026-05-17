use iced::{Element, Task};
use recovery_engine::types::ExtractedFile;

use crate::message::{Message, StrategyKind};
use crate::views;

/// Which screen is currently shown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    Scan,
    Results,
}

/// The complete application state.
pub struct App {
    pub screen: Screen,
    pub source_path: String,
    pub strategy: StrategyKind,
    pub files: Vec<ExtractedFile>,
    pub selected_file: Option<usize>,
    pub scanning: bool,
    pub error: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            screen: Screen::Scan,
            source_path: String::new(),
            strategy: StrategyKind::Carver,
            files: Vec::new(),
            selected_file: None,
            scanning: false,
            error: None,
        }
    }
}

impl App {
    /// Handles a message and returns any follow-up task (e.g., spawning a scan).
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SourcePathChanged(path) => {
                self.source_path = path;
                self.error = None;
                Task::none()
            }

            Message::StrategyChanged(strategy) => {
                self.strategy = strategy;
                Task::none()
            }

            Message::ScanPressed => {
                if self.source_path.trim().is_empty() {
                    self.error = Some("Enter a source path before scanning.".to_string());
                    return Task::none();
                }
                self.scanning = true;
                self.error = None;
                Task::perform(
                    crate::worker::run_scan(self.source_path.clone(), self.strategy),
                    Message::ScanCompleted,
                )
            }

            Message::ScanCompleted(result) => {
                self.scanning = false;
                match result {
                    Ok(files) => {
                        self.files = files;
                        self.selected_file = None;
                        self.screen = Screen::Results;
                    }
                    Err(err) => {
                        self.error = Some(err);
                    }
                }
                Task::none()
            }

            Message::FileSelected(index) => {
                self.selected_file = Some(index);
                Task::none()
            }

            Message::BackToScan => {
                self.screen = Screen::Scan;
                self.files.clear();
                self.selected_file = None;
                self.error = None;
                Task::none()
            }
        }
    }

    /// Renders the current screen.
    pub fn view(&self) -> Element<'_, Message> {
        match self.screen {
            Screen::Scan => views::scan::view(self),
            Screen::Results => views::results::view(self),
        }
    }
}

/// Formats a byte count into a human-readable string (B / KB / MB).
pub fn format_size(bytes: usize) -> String {
    if bytes < 1_024 {
        format!("{} B", bytes)
    } else if bytes < 1_024 * 1_024 {
        format!("{:.1} KB", bytes as f64 / 1_024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1_024.0 * 1_024.0))
    }
}
