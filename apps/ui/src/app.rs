use std::collections::HashSet;
use std::time::Duration;

use device_detector::StorageDevice;
use iced::widget::image::Handle as ImageHandle;
use iced::{Element, Subscription, Task, time};
use recovery_engine::types::ExtractedFile;

use crate::locale::{self, Locale, Strings};
use crate::log_capture::{LogBuffer, LogEntry};
use crate::message::{Message, StrategyKind};
use crate::notification::Notification;
use crate::screen::Screen;
use crate::views;

/// State of the export operation.
#[derive(Debug, Clone, PartialEq)]
pub enum ExportState {
    Idle,
    Picking,
    Exporting,
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
    pub export_state: ExportState,
    pub devices: Vec<StorageDevice>,
    pub detecting_devices: bool,
    pub locale: Locale,
    pub log_buffer: LogBuffer,
    pub log_entries: Vec<LogEntry>,
    pub sidebar_expanded: bool,
    pub console_open: bool,
    pub preview_handle: Option<ImageHandle>,
    pub notifications: Vec<Notification>,
    next_notification_id: u64,
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
            export_state: ExportState::Idle,
            devices: Vec::new(),
            detecting_devices: false,
            locale: Locale::detect(),
            log_buffer,
            log_entries: Vec::new(),
            sidebar_expanded: false,
            console_open: false,
            preview_handle: None,
            notifications: Vec::new(),
            next_notification_id: 0,
        }
    }

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
                Task::perform(
                    crate::worker::Worker::detect_devices(),
                    Message::DevicesDetected,
                )
            }

            Message::DevicesDetected(result) => {
                self.detecting_devices = false;
                match result {
                    Ok(devices) => self.devices = devices,
                    Err(err) => {
                        let id = self.next_id();
                        let msg = self.translations().toast_devices_error(err);
                        self.push_notification(Notification::error(id, msg));
                    }
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
                Task::none()
            }

            Message::StrategyChanged(strategy) => {
                self.strategy = strategy;
                Task::none()
            }

            Message::ScanPressed => {
                if self.source_path.trim().is_empty() {
                    let id = self.next_id();
                    let msg = self.translations().scan_error_no_source;
                    self.push_notification(Notification::error(id, msg));
                    return Task::none();
                }
                self.scanning = true;
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
                        let id = self.next_id();
                        let msg = self.translations().toast_scan_error(err);
                        self.push_notification(Notification::error(id, msg));
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
                Task::none()
            }

            Message::ToggleAll => {
                if self.selected_files.len() == self.files.len() {
                    self.selected_files.clear();
                } else {
                    self.selected_files = (0..self.files.len()).collect();
                }
                Task::none()
            }

            Message::ExportPressed => {
                self.export_state = ExportState::Picking;
                let title = self.translations().export_folder_title;
                Task::perform(
                    crate::worker::Worker::pick_folder(title),
                    Message::FolderPicked,
                )
            }

            Message::FolderPicked(None) => {
                self.export_state = ExportState::Idle;
                let id = self.next_id();
                let msg = self.translations().toast_export_cancelled;
                self.push_notification(Notification::info(id, msg));
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
                self.export_state = ExportState::Idle;
                self.selected_files.clear();
                let id = self.next_id();
                let msg = self.translations().toast_export_success(n);
                self.push_notification(Notification::success(id, msg));
                Task::none()
            }

            Message::ExportFailed(err) => {
                self.export_state = ExportState::Idle;
                let id = self.next_id();
                let msg = self.translations().toast_export_failed(err);
                self.push_notification(Notification::error(id, msg));
                Task::none()
            }

            // ── sidebar ───────────────────────────────────────────────────────
            Message::ToggleSidebar => {
                self.sidebar_expanded = !self.sidebar_expanded;
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
                self.notifications.retain(|n| !n.is_expired());
                Task::none()
            }

            // ── notifications ─────────────────────────────────────────────────
            Message::DismissNotification(id) => {
                self.notifications.retain(|n| n.id != id);
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

    pub fn all_selected(&self) -> bool {
        !self.files.is_empty() && self.selected_files.len() == self.files.len()
    }

    fn next_id(&mut self) -> u64 {
        let id = self.next_notification_id;
        self.next_notification_id += 1;
        id
    }

    fn push_notification(&mut self, notification: Notification) {
        self.notifications.push(notification);
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
