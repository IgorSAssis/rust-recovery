mod app;
mod log_capture;
mod message;
mod notification;
mod screen;
mod locale;
mod utils;
mod views;
mod worker;

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use app::App;
use log_capture::{LogBuffer, LogCaptureLayer};
use message::Message;

fn main() -> iced::Result {
    let buffer: LogBuffer = Arc::new(Mutex::new(VecDeque::new()));

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(LogCaptureLayer::new(buffer.clone()))
        .init();

    iced::application(move || boot(buffer.clone()), update, view)
        .title("RustRecover")
        .theme(theme)
        .subscription(subscription)
        .font(iced_fonts::BOOTSTRAP_FONT_BYTES)
        .run()
}

fn boot(buffer: LogBuffer) -> (App, iced::Task<Message>) {
    (App::new(buffer), iced::Task::none())
}

fn update(state: &mut App, message: Message) -> iced::Task<Message> {
    state.update(message)
}

fn theme(_state: &App) -> iced::Theme {
    iced::Theme::Dracula
}

fn view(state: &App) -> iced::Element<'_, Message> {
    state.view()
}

fn subscription(state: &App) -> iced::Subscription<Message> {
    state.subscription()
}
