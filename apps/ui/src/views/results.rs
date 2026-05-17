use iced::widget::image::{Handle as ImageHandle, Image as IcedImage};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, ContentFit, Element, Length};

use crate::app::{format_size, App};
use crate::message::{Message, StrategyKind};

pub fn view(app: &App) -> Element<'_, Message> {
    let header = build_header(app);
    let body = row![file_list(app), preview_panel(app)]
        .spacing(0)
        .height(Length::Fill);

    column![header, body]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(16)
        .into()
}

// ── Header ───────────────────────────────────────────────────────────────────

fn build_header(app: &App) -> Element<'_, Message> {
    let back_btn = button(text("← Back").size(14)).on_press(Message::BackToScan);
    let heading = text("Recovered Files").size(24);
    let strategy_label = match app.strategy {
        StrategyKind::Carver => "carver",
        StrategyKind::Fat32 => "fat32",
    };
    let summary = text(format!(
        "Found {} file(s) via {}.",
        app.files.len(),
        strategy_label,
    ))
    .size(13);

    column![
        row![back_btn, heading]
            .spacing(16)
            .align_y(Alignment::Center),
        summary,
        text(""),
    ]
    .spacing(6)
    .into()
}

// ── File list (left panel) ────────────────────────────────────────────────────

fn file_list(app: &App) -> Element<'_, Message> {
    let content: Element<Message> = if app.files.is_empty() {
        text("No recoverable files were found.").size(14).into()
    } else {
        let rows: Vec<Element<Message>> = app
            .files
            .iter()
            .enumerate()
            .map(|(i, file)| {
                let label = format!("{:<30} {}", file.filename, format_size(file.bytes.len()));
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

    container(content)
        .width(Length::FillPortion(1))
        .height(Length::Fill)
        .padding(8)
        .into()
}

// ── Preview panel (right panel) ───────────────────────────────────────────────

fn preview_panel(app: &App) -> Element<'_, Message> {
    let content: Element<Message> = match app.selected_file {
        None => container(text("Select a file to preview").size(14))
            .center(Length::Fill)
            .into(),

        Some(i) => {
            let file = &app.files[i];
            let is_image = matches!(file.extension.as_str(), "jpg" | "jpeg" | "png");

            if is_image {
                let handle = ImageHandle::from_bytes(file.bytes.clone());
                column![
                    IcedImage::new(handle)
                        .content_fit(ContentFit::Contain)
                        .width(Length::Fill)
                        .height(Length::Fill),
                    text(&file.filename).size(12),
                    text(format_size(file.bytes.len())).size(12),
                ]
                .spacing(8)
                .align_x(Alignment::Center)
                .into()
            } else {
                container(
                    column![
                        text("📄").size(52),
                        text(&file.filename).size(13),
                        text(format!(".{}", file.extension)).size(12),
                        text(format_size(file.bytes.len())).size(12),
                    ]
                    .spacing(8)
                    .align_x(Alignment::Center),
                )
                .center(Length::Fill)
                .into()
            }
        }
    };

    container(content)
        .width(Length::FillPortion(1))
        .height(Length::Fill)
        .padding(16)
        .into()
}

