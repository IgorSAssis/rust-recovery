use iced::widget::{button, column, container, radio, row, text, text_input};
use iced::{Alignment, Element, Length};
use iced_fonts::bootstrap;

use crate::app::App;
use crate::message::{Message, StrategyKind};

pub fn view(app: &App) -> Element<'_, Message> {
    let translations = app.translations();

    let source_label = row![
        bootstrap::hdd().size(14),
        text(translations.source_label).size(14),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    let source_input = text_input(translations.source_placeholder, &app.source_path)
        .on_input(Message::SourcePathChanged)
        .padding(10);

    let strategy_label = text(translations.strategy_label);
    let carver_radio = radio(
        translations.strategy_carver,
        StrategyKind::Carver,
        Some(app.strategy),
        Message::StrategyChanged,
    );
    let fat32_radio = radio(
        translations.strategy_fat32,
        StrategyKind::Fat32,
        Some(app.strategy),
        Message::StrategyChanged,
    );

    let status: Element<Message> = if app.scanning {
        text(translations.scanning_status).size(14).into()
    } else {
        text("").size(14).into()
    };

    let scan_btn = if app.scanning {
        button(
            row![
                bootstrap::hourglass_split().size(14),
                text(translations.scanning_btn).size(16)
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
    } else {
        button(
            row![
                bootstrap::search().size(14),
                text(translations.scan_btn).size(16)
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .on_press(Message::ScanPressed)
    };

    let content = column![
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

    container(content).center(Length::Fill).padding(40).into()
}
