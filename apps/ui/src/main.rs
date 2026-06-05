mod app;
mod message;
mod screen;
mod utils;
mod views;
mod worker;

use app::App;
use message::Message;

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::application(boot, update, view)
        .title("RustRecover")
        .theme(theme)
        .run()
}

fn boot() -> (App, iced::Task<Message>) {
    (App::default(), iced::Task::none())
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
