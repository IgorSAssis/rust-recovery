use iced::widget::image::Image as IcedImage;
use iced::widget::{
    button, checkbox, column, container, progress_bar, row, rule, scrollable, text,
};
use iced::{Alignment, Color, ContentFit, Element, Length};
use iced_fonts::bootstrap;

use crate::app::{App, ExportState};
use crate::message::{Message, StrategyKind};
use crate::utils::format_size;

pub fn view(app: &App) -> Element<'_, Message> {
    let left = container(
        column![build_summary(app), build_export_bar(app), file_list(app)]
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .width(Length::FillPortion(1))
    .height(Length::Fill)
    .padding(16);

    row![left, rule::vertical(1), preview_panel(app)]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

// ── Summary ───────────────────────────────────────────────────────────────────

fn build_summary(app: &App) -> Element<'_, Message> {
    let translations = app.translations();
    let strategy_label = match app.strategy {
        StrategyKind::Carver => "carver",
        StrategyKind::Fat32 => "fat32",
    };
    text(translations.files_found(app.files.len(), strategy_label))
        .size(13)
        .into()
}

// ── Export bar ────────────────────────────────────────────────────────────────

fn build_export_bar(app: &App) -> Element<'_, Message> {
    let translations = app.translations();
    let n_selected = app.selected_files.len();
    let is_busy = matches!(
        app.export_state,
        ExportState::Picking | ExportState::Exporting
    );

    let (toggle_icon, toggle_label) = if app.all_selected() {
        (bootstrap::dash_square().size(13), translations.deselect_all)
    } else {
        (bootstrap::check_square().size(13), translations.select_all)
    };
    let toggle_btn = button(
        row![toggle_icon, text(toggle_label).size(13)]
            .spacing(6)
            .align_y(Alignment::Center),
    )
    .style(button::secondary)
    .on_press(Message::ToggleAll);

    let export_btn = {
        let label = translations.export_selected(n_selected);
        let content = row![bootstrap::download().size(13), text(label).size(13)]
            .spacing(6)
            .align_y(Alignment::Center);
        let b = button(content).style(button::primary);
        if n_selected > 0 && !is_busy {
            b.on_press(Message::ExportPressed)
        } else {
            b
        }
    };

    let status: Element<Message> = match &app.export_state {
        ExportState::Idle => text("").size(13).into(),
        ExportState::Picking => row![
            bootstrap::foldertwo_open().size(13),
            text(translations.export_picking).size(13),
        ]
        .spacing(6)
        .align_y(Alignment::Center)
        .into(),
        ExportState::Exporting => column![
            row![
                bootstrap::arrow_repeat().size(13),
                text(translations.exporting).size(13),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
            container(progress_bar(0.0..=1.0_f32, 0.5_f32)).width(200),
        ]
        .spacing(4)
        .into(),
        ExportState::Done(n) => row![
            bootstrap::check_circle().size(13).color(Color::from_rgb(0.2, 0.8, 0.2)),
            text(translations.export_success(*n))
                .size(13)
                .color(Color::from_rgb(0.2, 0.8, 0.2)),
        ]
        .spacing(6)
        .align_y(Alignment::Center)
        .into(),
        ExportState::Failed(err) => row![
            bootstrap::x_circle().size(13).color(Color::from_rgb(0.9, 0.2, 0.2)),
            text(translations.export_error(err))
                .size(13)
                .color(Color::from_rgb(0.9, 0.2, 0.2)),
        ]
        .spacing(6)
        .align_y(Alignment::Center)
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
    let translations = app.translations();

    let content: Element<Message> = if app.files.is_empty() {
        text(translations.no_files_found).size(14).into()
    } else {
        let rows: Vec<Element<Message>> = app
            .files
            .iter()
            .enumerate()
            .map(|(i, file)| {
                let is_checked = app.selected_files.contains(&i);
                let is_selected = app.selected_file == Some(i);

                let file_icon = if file.is_image() {
                    bootstrap::file_earmark_image().size(13)
                } else {
                    bootstrap::file_earmark().size(13)
                };

                let label = format!("{:<28} {}", file.filename, format_size(file.bytes.len()));
                let file_btn = if is_selected {
                    button(
                        row![file_icon, text(label).size(13)]
                            .spacing(6)
                            .align_y(Alignment::Center),
                    )
                    .style(button::primary)
                    .width(Length::Fill)
                } else {
                    button(
                        row![file_icon, text(label).size(13)]
                            .spacing(6)
                            .align_y(Alignment::Center),
                    )
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
    let translations = app.translations();

    let content: Element<Message> = match app.selected_file {
        None => container(text(translations.select_to_preview).size(14))
            .center(Length::Fill)
            .into(),

        Some(i) => {
            let file = &app.files[i];

            match &app.preview_handle {
                Some(handle) => column![
                    IcedImage::new(handle.clone())
                        .content_fit(ContentFit::Contain)
                        .width(Length::Fill)
                        .height(Length::Fill),
                    text(&file.filename).size(12),
                    text(format_size(file.bytes.len())).size(12),
                ]
                .spacing(8)
                .align_x(Alignment::Center)
                .into(),

                None => container(
                    column![
                        bootstrap::file_earmark().size(52),
                        text(&file.filename).size(13),
                        text(format!(".{}", file.extension)).size(12),
                        text(format_size(file.bytes.len())).size(12),
                    ]
                    .spacing(8)
                    .align_x(Alignment::Center),
                )
                .center(Length::Fill)
                .into(),
            }
        }
    };

    container(content)
        .width(Length::FillPortion(2))
        .height(Length::Fill)
        .padding(16)
        .into()
}
