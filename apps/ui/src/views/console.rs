use std::time::UNIX_EPOCH;

use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Color, Element, Length};
use iced_fonts::bootstrap;
use tracing::Level;

use crate::app::App;
use crate::message::Message;

const CONSOLE_HEIGHT: f32 = 220.0;

pub fn view(app: &App) -> Element<'_, Message> {
    let translations = app.translations();

    let (toggle_icon, toggle_label) = if app.console_open {
        (bootstrap::chevron_down().size(12), translations.console_toggle_close)
    } else {
        (bootstrap::chevron_up().size(12), translations.console_toggle_open)
    };

    let toggle_btn = button(
        row![toggle_icon, text(toggle_label).size(12)]
            .spacing(6)
            .align_y(Alignment::Center),
    )
    .style(button::text)
    .on_press(Message::ToggleConsole);

    let header = container(
        row![
            bootstrap::terminal().size(12),
            text(translations.console_title).size(12),
            iced::widget::Space::new().width(Length::Fill),
            toggle_btn,
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    )
    .padding([4, 12])
    .width(Length::Fill);

    if !app.console_open {
        return container(header).width(Length::Fill).into();
    }

    let body: Element<Message> = if app.log_entries.is_empty() {
        container(text(translations.console_no_logs).size(12))
            .padding([4, 12])
            .width(Length::Fill)
            .into()
    } else {
        let rows: Vec<Element<Message>> = app
            .log_entries
            .iter()
            .map(|entry| entry_row(entry))
            .collect();

        scrollable(
            column(rows)
                .spacing(2)
                .padding([4, 12])
                .width(Length::Fill),
        )
        .height(CONSOLE_HEIGHT)
        .anchor_bottom()
        .into()
    };

    container(column![header, body]).width(Length::Fill).into()
}

fn entry_row(entry: &crate::log_capture::LogEntry) -> Element<'_, Message> {
    let level_color = match entry.level {
        Level::ERROR => Color::from_rgb(0.9, 0.3, 0.3),
        Level::WARN => Color::from_rgb(0.95, 0.75, 0.2),
        Level::INFO => Color::from_rgb(0.4, 0.85, 0.5),
        Level::DEBUG => Color::from_rgb(0.6, 0.6, 0.6),
        Level::TRACE => Color::from_rgb(0.4, 0.4, 0.4),
    };

    let level_icon = match entry.level {
        Level::ERROR => bootstrap::x_circle().size(11).color(level_color),
        Level::WARN => bootstrap::exclamation_triangle().size(11).color(level_color),
        Level::INFO => bootstrap::info_circle().size(11).color(level_color),
        Level::DEBUG => bootstrap::bug().size(11).color(level_color),
        Level::TRACE => bootstrap::dot().size(11).color(level_color),
    };

    let timestamp = entry
        .timestamp
        .duration_since(UNIX_EPOCH)
        .map(|d| {
            let secs = d.as_secs();
            let h = (secs % 86400) / 3600;
            let m = (secs % 3600) / 60;
            let s = secs % 60;
            format!("{:02}:{:02}:{:02}", h, m, s)
        })
        .unwrap_or_else(|_| "--:--:--".to_string());

    row![
        text(timestamp).size(11).color(Color::from_rgb(0.5, 0.5, 0.5)),
        level_icon,
        text(format!("{} — {}", entry.target, entry.message)).size(11),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}
