use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Length};

use crate::app::{format_size, App};
use crate::message::Message;

pub fn view(app: &App) -> Element<'_, Message> {
    let back_btn = button(text("← Back").size(14)).on_press(Message::BackToScan);
    let heading = text("Recovered Files").size(24);

    let header = row![back_btn, heading]
        .spacing(16)
        .align_y(Alignment::Center);

    let strategy_label = match app.strategy {
        crate::message::StrategyKind::Carver => "carver (signature-based)",
        crate::message::StrategyKind::Fat32 => "fat32 (filesystem-aware)",
    };
    let summary = text(format!(
        "Found {} file(s) via {}.",
        app.files.len(),
        strategy_label
    ))
    .size(14);

    let file_list: Element<Message> = if app.files.is_empty() {
        text("No recoverable files were found.").size(14).into()
    } else {
        let rows: Vec<Element<Message>> = app
            .files
            .iter()
            .enumerate()
            .map(|(i, file)| {
                let label = format!(
                    "{}    {}",
                    file.filename,
                    format_size(file.bytes.len()),
                );
                let is_selected = app.selected_file == Some(i);
                let btn = if is_selected {
                    button(text(label).size(13)).style(button::primary)
                } else {
                    button(text(label).size(13)).style(button::secondary)
                };
                btn.on_press(Message::FileSelected(i))
                    .width(Length::Fill)
                    .into()
            })
            .collect();

        scrollable(column(rows).spacing(4).width(Length::Fill))
            .height(Length::Fill)
            .into()
    };

    let content = column![header, summary, file_list]
        .spacing(16)
        .padding(24)
        .width(Length::Fill)
        .height(Length::Fill);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
