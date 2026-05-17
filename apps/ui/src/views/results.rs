use iced::widget::image::{Handle as ImageHandle, Image as IcedImage};
use iced::widget::{button, checkbox, column, container, progress_bar, row, scrollable, text};
use iced::{Alignment, Color, ContentFit, Element, Length};

use crate::app::{format_size, App};
use crate::message::{ExportState, Message, StrategyKind};

pub fn view(app: &App) -> Element<'_, Message> {
    let header = build_header(app);
    let export_bar = build_export_bar(app);
    let body = row![file_list(app), preview_panel(app)]
        .spacing(0)
        .height(Length::Fill);

    column![header, export_bar, body]
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

// ── Export bar ────────────────────────────────────────────────────────────────

fn build_export_bar(app: &App) -> Element<'_, Message> {
    let n_selected = app.selected_files.len();
    let is_busy = matches!(
        app.export_state,
        ExportState::Picking | ExportState::Exporting
    );

    let toggle_label = if app.all_selected() {
        "Deselect All"
    } else {
        "Select All"
    };
    let toggle_btn = button(text(toggle_label).size(13))
        .style(button::secondary)
        .on_press(Message::ToggleAll);

    let export_btn = {
        let label = format!("Export Selected ({})", n_selected);
        let b = button(text(label).size(13)).style(button::primary);
        if n_selected > 0 && !is_busy {
            b.on_press(Message::ExportPressed)
        } else {
            b
        }
    };

    let status: Element<Message> = match &app.export_state {
        ExportState::Idle => text("").size(13).into(),
        ExportState::Picking => text("Opening folder picker…").size(13).into(),
        ExportState::Exporting => column![
            text("Exporting…").size(13),
            container(progress_bar(0.0..=1.0_f32, 0.5_f32)).width(200),
        ]
        .spacing(4)
        .into(),
        ExportState::Done(n) => text(format!("✓ {} file(s) exported successfully.", n))
            .size(13)
            .color(Color::from_rgb(0.2, 0.8, 0.2))
            .into(),
        ExportState::Failed(err) => text(format!("✗ {}", err))
            .size(13)
            .color(Color::from_rgb(0.9, 0.2, 0.2))
            .into(),
    };

    container(
        row![toggle_btn, export_btn, status]
            .spacing(12)
            .align_y(Alignment::Center),
    )
    .padding([8, 0])
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
                let is_checked = app.selected_files.contains(&i);
                let is_selected = app.selected_file == Some(i);

                let label = format!("{:<28} {}", file.filename, format_size(file.bytes.len()));
                let file_btn = if is_selected {
                    button(text(label).size(13))
                        .style(button::primary)
                        .width(Length::Fill)
                } else {
                    button(text(label).size(13))
                        .style(button::secondary)
                        .width(Length::Fill)
                };

                row![
                    checkbox(is_checked).on_toggle(move |_| Message::FileToggled(i)),
                    file_btn.on_press(Message::FileSelected(i)),
                ]
                .spacing(8)
                .align_y(Alignment::Center)
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
