use iced::widget::{button, column, container, row, rule, text};
use iced::{Alignment, Element, Length};

use crate::app::App;
use crate::message::Message;
use crate::screen::Screen;

use super::{devices, results, scan};

pub fn view(app: &App) -> Element<'_, Message> {
    let content = match app.screen {
        Screen::Devices => devices::view(app),
        Screen::Scan => scan::view(app),
        Screen::Results => results::view(app),
    };

    column![
        header(),
        rule::horizontal(1),
        row![
            sidebar(app),
            rule::vertical(1),
            container(content).width(Length::Fill).height(Length::Fill),
        ]
        .height(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn header<'a>() -> Element<'a, Message> {
    container(
        column![
            text("RustRecover").size(20),
            text("File Recovery Tool").size(13),
        ]
        .align_x(Alignment::Start),
    )
    .padding([12, 16])
    .width(Length::Fill)
    .into()
}

fn sidebar(app: &App) -> Element<'_, Message> {
    let nav_btn = |label, screen: Screen| {
        let b = button(text(label).size(14)).width(Length::Fill);
        if app.screen == screen {
            b.style(button::primary)
                .on_press(Message::NavigateTo(screen))
        } else {
            b.style(button::text).on_press(Message::NavigateTo(screen))
        }
    };

    let recover_btn = {
        let style = if app.screen == Screen::Results {
            button::primary
        } else {
            button::text
        };
        let b = button(text("Recuperar").size(14))
            .width(Length::Fill)
            .style(style);
        if !app.files.is_empty() {
            b.on_press(Message::NavigateTo(Screen::Results))
        } else {
            b
        }
    };

    container(
        column![
            nav_btn("Dispositivos", Screen::Devices),
            nav_btn("Escanear", Screen::Scan),
            recover_btn,
        ]
        .spacing(4),
    )
    .width(220)
    .height(Length::Fill)
    .padding(16)
    .into()
}
