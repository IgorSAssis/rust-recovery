use iced::widget::{button, column, container, pick_list, row, rule, text};
use iced::{Alignment, Element, Length};

use crate::app::App;
use crate::locale::Locale;
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
        header(app),
        rule::horizontal(1),
        row![
            sidebar(app),
            rule::vertical(1),
            container(content)
                .width(Length::Fill)
                .height(Length::Fill),
        ]
        .height(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn header(app: &App) -> Element<'_, Message> {
    let translations = app.translations();

    let lang_selector = pick_list(Locale::ALL, Some(app.locale), Message::LanguageChanged);

    container(
        row![
            column![
                text("RustRecover").size(20),
                text(translations.app_subtitle).size(13),
            ]
            .spacing(2),
            iced::widget::Space::new().width(Length::Fill),
            lang_selector,
        ]
        .align_y(Alignment::Center),
    )
    .padding([12, 16])
    .width(Length::Fill)
    .into()
}

fn sidebar(app: &App) -> Element<'_, Message> {
    let translations = app.translations();

    let nav_btn = |label, screen: Screen| {
        let b = button(text(label).size(14)).width(Length::Fill);
        if app.screen == screen {
            b.style(button::primary).on_press(Message::NavigateTo(screen))
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
        let b = button(text(translations.nav_recover).size(14))
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
            nav_btn(translations.nav_devices, Screen::Devices),
            nav_btn(translations.nav_scan, Screen::Scan),
            recover_btn,
        ]
        .spacing(4),
    )
    .width(220)
    .height(Length::Fill)
    .padding(16)
    .into()
}
