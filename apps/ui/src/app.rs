use std::collections::HashSet;
use std::time::Duration;

use device_detector::StorageDevice;
use iced::widget::image::Handle as ImageHandle;
use iced::{Element, Subscription, Task, time};
use recovery_engine::types::ExtractedFile;

use crate::log_capture::{LogBuffer, LogEntry};
use crate::message::{Message, StrategyKind};
use crate::screen::Screen;
use crate::locale::{self, Locale, Strings};
use crate::views;

/// State of the export operation.
#[derive(Debug, Clone, PartialEq)]
pub enum ExportState {
    Idle,
    Picking,
    Exporting,
    Done(usize),
    Failed(String),
}

/// The complete application state.
#[derive(Clone)]
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
    pub devices: Vec<StorageDevice>,
    pub detecting_devices: bool,
    pub locale: Locale,
    pub log_buffer: LogBuffer,
    pub log_entries: Vec<LogEntry>,
    pub console_open: bool,
    /// Cached image handle for the currently selected file.
    /// Created once on FileSelected and reused across re-renders to avoid
    /// uploading a new GPU texture on every view() call.
    pub preview_handle: Option<ImageHandle>,
}

impl App {
    pub fn new(log_buffer: LogBuffer) -> Self {
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
            devices: Vec::new(),
            detecting_devices: false,
            locale: Locale::detect(),
            log_buffer,
            log_entries: Vec::new(),
            console_open: false,
            preview_handle: None,
        }
    }

    /// Handles a message and returns any follow-up task (e.g., spawning a scan).
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // ── navigation ────────────────────────────────────────────────────
            Message::NavigateTo(screen) => {
                self.screen = screen;
                Task::none()
            }

            Message::LanguageChanged(locale) => {
                self.locale = locale;
                Task::none()
            }

            // ── devices ───────────────────────────────────────────────────────
            Message::DetectDevicesPressed => {
                self.detecting_devices = true;
                Task::perform(crate::worker::Worker::detect_devices(), Message::DevicesDetected)
            }

            Message::DevicesDetected(result) => {
                self.detecting_devices = false;
                match result {
                    Ok(devices) => self.devices = devices,
                    Err(err) => self.error = Some(err),
                }
                Task::none()
            }

            Message::DeviceSelected(path) => {
                self.source_path = path.to_string_lossy().to_string();
                self.screen = Screen::Scan;
                Task::none()
            }

            // ── scan ──────────────────────────────────────────────────────────
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
                    self.error = Some(self.translations().scan_error_no_source.to_string());
                    return Task::none();
                }
                self.scanning = true;
                self.error = None;
                Task::perform(
                    crate::worker::Worker::run_scan(self.source_path.clone(), self.strategy),
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
                        self.preview_handle = None;
                        self.screen = Screen::Results;
                    }
                    Err(err) => {
                        self.error = Some(err);
                    }
                }
                Task::none()
            }

            // ── results ───────────────────────────────────────────────────────
            Message::FileSelected(index) => {
                self.selected_file = Some(index);
                self.preview_handle = self.build_preview_handle(index);
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
                let title = self.translations().export_folder_title;
                Task::perform(crate::worker::Worker::pick_folder(title), Message::FolderPicked)
            }

            Message::FolderPicked(None) => {
                self.export_state = ExportState::Idle;
                Task::none()
            }

            Message::FolderPicked(Some(path)) => {
                self.export_state = ExportState::Exporting;
                let files = self.files.clone();
                let selected = self.selected_files.clone();
                Task::perform(
                    crate::worker::Worker::run_export(files, selected, path),
                    |result| match result {
                        Ok(n) => Message::ExportCompleted(n),
                        Err(e) => Message::ExportFailed(e),
                    },
                )
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

            // ── console ───────────────────────────────────────────────────────
            Message::ToggleConsole => {
                self.console_open = !self.console_open;
                Task::none()
            }

            Message::LogDrainTick => {
                if let Ok(mut buf) = self.log_buffer.lock() {
                    self.log_entries.extend(buf.drain(..));
                }
                Task::none()
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_millis(200)).map(|_| Message::LogDrainTick)
    }

    pub fn translations(&self) -> &'static Strings {
        match self.locale {
            Locale::PtBr => &locale::PT_BR,
            Locale::En => &locale::EN,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        views::app_container::view(self)
    }

    /// Returns true if all files are currently selected.
    pub fn all_selected(&self) -> bool {
        !self.files.is_empty() && self.selected_files.len() == self.files.len()
    }

    fn build_preview_handle(&self, index: usize) -> Option<ImageHandle> {
        let file = self.files.get(index)?;
        if file.is_image() {
            Some(ImageHandle::from_bytes(file.bytes.clone()))
        } else {
            None
        }
    }
}
