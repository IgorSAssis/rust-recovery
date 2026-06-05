use iced::border::Radius;
use iced::widget::{button, column, container, pick_list, row, rule, stack, text};
use iced::{Alignment, Background, Border, Element, Length};
use iced_fonts::bootstrap;

use crate::app::App;
use crate::locale::LocaleOption;
use crate::message::Message;
use crate::screen::Screen;

use super::{console, devices, results, scan, toast};

const SIDEBAR_EXPANDED_WIDTH: f32 = 220.0;
const SIDEBAR_COLLAPSED_WIDTH: f32 = 56.0;
const TOGGLE_BTN_SIZE: f32 = 22.0;

pub fn view(app: &App) -> Element<'_, Message> {
    let screen_content = match app.screen {
        Screen::Devices => devices::view(app),
        Screen::Scan => scan::view(app),
        Screen::Results => results::view(app),
    };

    let sidebar_width = if app.sidebar_expanded {
        SIDEBAR_EXPANDED_WIDTH
    } else {
        SIDEBAR_COLLAPSED_WIDTH
    };

    let main_row = row![
        sidebar(app),
        rule::vertical(1),
        container(screen_content)
            .width(Length::Fill)
            .height(Length::Fill),
    ]
    .height(Length::Fill);

    let main_area = stack![main_row, sidebar_toggle_overlay(app, sidebar_width)]
        .width(Length::Fill)
        .height(Length::Fill);

    let layout = column![
        header(app),
        rule::horizontal(1),
        main_area,
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

/// Creates the toggle button overlay, centered vertically on the sidebar divider.
fn sidebar_toggle_overlay(app: &App, sidebar_width: f32) -> Element<'_, Message> {
    let toggle_icon = if app.sidebar_expanded {
        bootstrap::chevron_double_left().size(11)
    } else {
        bootstrap::chevron_double_right().size(11)
    };

    let toggle_btn = button(container(toggle_icon).center(Length::Fill))
        .width(TOGGLE_BTN_SIZE)
        .height(TOGGLE_BTN_SIZE)
        .padding(0)
        .style(|theme: &iced::Theme, status| {
            let palette = theme.extended_palette();
            let bg_color = match status {
                button::Status::Hovered | button::Status::Pressed => {
                    palette.background.weak.color
                }
                _ => palette.background.base.color,
            };
            button::Style {
                background: Some(Background::Color(bg_color)),
                border: Border {
                    color: palette.background.strong.color,
                    width: 1.0,
                    radius: Radius::new(TOGGLE_BTN_SIZE / 2.0),
                },
                text_color: palette.background.base.text,
                ..Default::default()
            }
        })
        .on_press(Message::ToggleSidebar);

    // Position the button so its center aligns with the divider line.
    // A left spacer of (sidebar_width - half_button_width) places the button
    // straddling the rule::vertical that sits right after the sidebar.
    let left_spacer_width = sidebar_width - (TOGGLE_BTN_SIZE / 2.0);

    container(
        row![
            iced::widget::Space::new().width(left_spacer_width),
            toggle_btn,
        ]
    )
    .align_y(iced::alignment::Vertical::Center)
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

const NAV_BTN_ICON_SIZE: u32 = 34;

fn nav_btn<'a>(
    icon: iced::widget::Text<'a>,
    label: &'a str,
    screen: Screen,
    app: &'a App,
    enabled: bool,
) -> Element<'a, Message> {
    let is_active = app.screen == screen;
    let btn_style = if is_active { button::primary } else { button::text };

    if app.sidebar_expanded {
        let content = row![icon, text(label).size(14)]
            .spacing(8)
            .align_y(Alignment::Center);

        let nav_button = button(content).width(Length::Fill).style(btn_style);

        if enabled {
            nav_button.on_press(Message::NavigateTo(screen)).into()
        } else {
            nav_button.into()
        }
    } else {
        let nav_button = button(container(icon).center(Length::Fill))
            .width(NAV_BTN_ICON_SIZE)
            .height(NAV_BTN_ICON_SIZE)
            .padding(0)
            .style(btn_style);

        let pressable_button = if enabled {
            nav_button.on_press(Message::NavigateTo(screen))
        } else {
            nav_button
        };

        container(pressable_button)
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .into()
    }
}

fn sidebar(app: &App) -> Element<'_, Message> {
    let translations = app.translations();

    let devices_btn = nav_btn(
        bootstrap::hdd().size(15),
        translations.nav_devices,
        Screen::Devices,
        app,
        true,
    );

    let scan_btn = nav_btn(
        bootstrap::search().size(15),
        translations.nav_scan,
        Screen::Scan,
        app,
        true,
    );

    let recover_btn = nav_btn(
        bootstrap::file_earmark_arrow_down().size(15),
        translations.nav_recover,
        Screen::Results,
        app,
        !app.files.is_empty(),
    );

    let nav_items = column![devices_btn, scan_btn, recover_btn]
        .spacing(4)
        .width(Length::Fill);

    let (sidebar_width, sidebar_padding) = if app.sidebar_expanded {
        (SIDEBAR_EXPANDED_WIDTH as u32, 12)
    } else {
        (SIDEBAR_COLLAPSED_WIDTH as u32, 8)
    };

    container(nav_items)
        .width(sidebar_width)
        .height(Length::Fill)
        .padding(sidebar_padding)
        .into()
}
