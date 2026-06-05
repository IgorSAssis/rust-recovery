use iced::widget::{button, column, container, pick_list, row, rule, stack, text};
use iced::{Alignment, Element, Length};
use iced_fonts::bootstrap;

use crate::app::App;
use crate::locale::LocaleOption;
use crate::message::Message;
use crate::screen::Screen;

use super::{console, devices, results, scan, toast};

pub fn view(app: &App) -> Element<'_, Message> {
    let content = match app.screen {
        Screen::Devices => devices::view(app),
        Screen::Scan => scan::view(app),
        Screen::Results => results::view(app),
    };

    let layout = column![
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
        rule::horizontal(1),
        console::view(app),
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    stack![layout, toast::view(app)]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn header(app: &App) -> Element<'_, Message> {
    let translations = app.translations();

    let locale_options = translations.locale_options();
    let selected_locale = LocaleOption {
        locale: app.locale,
        label: translations.locale_label(app.locale),
    };
    let lang_selector = pick_list(
        locale_options,
        Some(selected_locale),
        |opt: LocaleOption| Message::LanguageChanged(opt.locale),
    );

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

    let devices_btn = {
        let content = row![bootstrap::hdd().size(15), text(translations.nav_devices).size(14)]
            .spacing(8)
            .align_y(Alignment::Center);
        let b = button(content).width(Length::Fill);
        if app.screen == Screen::Devices {
            b.style(button::primary).on_press(Message::NavigateTo(Screen::Devices))
        } else {
            b.style(button::text).on_press(Message::NavigateTo(Screen::Devices))
        }
    };

    let scan_btn = {
        let content = row![bootstrap::search().size(15), text(translations.nav_scan).size(14)]
            .spacing(8)
            .align_y(Alignment::Center);
        let b = button(content).width(Length::Fill);
        if app.screen == Screen::Scan {
            b.style(button::primary).on_press(Message::NavigateTo(Screen::Scan))
        } else {
            b.style(button::text).on_press(Message::NavigateTo(Screen::Scan))
        }
    };

    let recover_btn = {
        let style = if app.screen == Screen::Results {
            button::primary
        } else {
            button::text
        };
        let content = row![
            bootstrap::file_earmark_arrow_down().size(15),
            text(translations.nav_recover).size(14)
        ]
        .spacing(8)
        .align_y(Alignment::Center);
        let b = button(content).width(Length::Fill).style(style);
        if !app.files.is_empty() {
            b.on_press(Message::NavigateTo(Screen::Results))
        } else {
            b
        }
    };

    container(
        column![devices_btn, scan_btn, recover_btn].spacing(4),
    )
    .width(220)
    .height(Length::Fill)
    .padding(16)
    .into()
}
