use iced::widget::{button, column, container, radio, row, text, text_input};
use iced::{Alignment, Element, Length};

use crate::app::App;
use crate::message::{Message, StrategyKind};

pub fn view(app: &App) -> Element<'_, Message> {
    let title = text("RustRecover").size(32);
    let subtitle = text("File Recovery Tool").size(14);

    let source_label = text("Source path:");
    let source_input = text_input(
        "/dev/sdb1  or  /path/to/image.img",
        &app.source_path,
    )
    .on_input(Message::SourcePathChanged)
    .padding(10);

    let strategy_label = text("Recovery strategy:");
    let carver_radio = radio(
        "Carver — signature-based (any device)",
        StrategyKind::Carver,
        Some(app.strategy),
        Message::StrategyChanged,
    );
    let fat32_radio = radio(
        "FAT32 — filesystem-aware (preserves filenames)",
        StrategyKind::Fat32,
        Some(app.strategy),
        Message::StrategyChanged,
    );

    let status: Element<Message> = if app.scanning {
        text("Scanning… this may take a while.").size(14).into()
    } else if let Some(err) = &app.error {
        text(err.as_str()).size(14).into()
    } else {
        text("").size(14).into()
    };

    let scan_btn = if app.scanning {
        button(text("Scanning…").size(16))
    } else {
        button(text("Scan").size(16)).on_press(Message::ScanPressed)
    };

    let content = column![
        title,
        subtitle,
        text(""),
        source_label,
        source_input,
        text(""),
        strategy_label,
        carver_radio,
        fat32_radio,
        text(""),
        status,
        row![scan_btn].align_y(Alignment::Center),
    ]
    .spacing(8)
    .max_width(600);

    container(content)
        .center(Length::Fill)
        .padding(40)
        .into()
}
