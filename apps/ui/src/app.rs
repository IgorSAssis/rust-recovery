use std::collections::HashSet;

use iced::{Element, Task};
use recovery_engine::types::ExtractedFile;

use crate::message::{ExportState, Message, StrategyKind};
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
    pub selected_files: HashSet<usize>,
    pub scanning: bool,
    pub error: Option<String>,
    pub export_state: ExportState,
}

impl Default for App {
    fn default() -> Self {
        Self {
            screen: Screen::Scan,
            source_path: String::new(),
            strategy: StrategyKind::Carver,
            files: Vec::new(),
            selected_file: None,
            selected_files: HashSet::new(),
            scanning: false,
            error: None,
            export_state: ExportState::Idle,
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
                        self.selected_files.clear();
                        self.export_state = ExportState::Idle;
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

            Message::FileToggled(index) => {
                if self.selected_files.contains(&index) {
                    self.selected_files.remove(&index);
                } else {
                    self.selected_files.insert(index);
                }
                self.export_state = ExportState::Idle;
                Task::none()
            }

            Message::ToggleAll => {
                if self.selected_files.len() == self.files.len() {
                    self.selected_files.clear();
                } else {
                    self.selected_files = (0..self.files.len()).collect();
                }
                self.export_state = ExportState::Idle;
                Task::none()
            }

            Message::ExportPressed => {
                self.export_state = ExportState::Picking;
                Task::perform(crate::worker::pick_folder(), Message::FolderPicked)
            }

            Message::FolderPicked(None) => {
                self.export_state = ExportState::Idle;
                Task::none()
            }

            Message::FolderPicked(Some(path)) => {
                self.export_state = ExportState::Exporting;
                let files = self.files.clone();
                let selected = self.selected_files.clone();
                Task::perform(crate::worker::run_export(files, selected, path), |result| {
                    match result {
                        Ok(n) => Message::ExportCompleted(n),
                        Err(e) => Message::ExportFailed(e),
                    }
                })
            }

            Message::ExportCompleted(n) => {
                self.export_state = ExportState::Done(n);
                self.selected_files.clear();
                Task::none()
            }

            Message::ExportFailed(err) => {
                self.export_state = ExportState::Failed(err);
                Task::none()
            }

            Message::BackToScan => {
                self.screen = Screen::Scan;
                self.files.clear();
                self.selected_file = None;
                self.selected_files.clear();
                self.export_state = ExportState::Idle;
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

    /// Returns true if all files are currently selected.
    pub fn all_selected(&self) -> bool {
        !self.files.is_empty() && self.selected_files.len() == self.files.len()
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

/// Needed so that `PathBuf` can appear in a Message clone.
impl Clone for App {
    fn clone(&self) -> Self {
        Self {
            screen: self.screen.clone(),
            source_path: self.source_path.clone(),
            strategy: self.strategy,
            files: self.files.clone(),
            selected_file: self.selected_file,
            selected_files: self.selected_files.clone(),
            scanning: self.scanning,
            error: self.error.clone(),
            export_state: self.export_state.clone(),
        }
    }
}
