use iced::border::Radius;
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Border, Color, Element, Length, Padding};
use iced_fonts::bootstrap;

use crate::app::App;
use crate::message::Message;
use crate::notification::NotificationKind;

const TOAST_WIDTH: f32 = 320.0;
const ACCENT_WIDTH: f32 = 4.0;

pub fn view(app: &App) -> Element<'_, Message> {
    if app.notifications.is_empty() {
        return iced::widget::Space::new().into();
    }

    let cards: Vec<Element<Message>> = app.notifications.iter().map(toast_card).collect();

    container(column(cards).spacing(8))
        .align_x(iced::alignment::Horizontal::Right)
        .align_y(iced::alignment::Vertical::Bottom)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding {
            top: 0.0,
            right: 16.0,
            bottom: 16.0,
            left: 0.0,
        })
        .into()
}

fn toast_card(notification: &crate::notification::Notification) -> Element<'_, Message> {
    let (accent_color, icon) = match notification.kind {
        NotificationKind::Success => (
            Color::from_rgb(0.2, 0.75, 0.35),
            bootstrap::check_circle()
                .size(15)
                .color(Color::from_rgb(0.2, 0.75, 0.35)),
        ),
        NotificationKind::Error => (
            Color::from_rgb(0.85, 0.25, 0.25),
            bootstrap::x_circle()
                .size(15)
                .color(Color::from_rgb(0.85, 0.25, 0.25)),
        ),
        NotificationKind::Warning => (
            Color::from_rgb(0.9, 0.65, 0.1),
            bootstrap::exclamation_triangle()
                .size(15)
                .color(Color::from_rgb(0.9, 0.65, 0.1)),
        ),
        NotificationKind::Info => (
            Color::from_rgb(0.3, 0.6, 0.95),
            bootstrap::info_circle()
                .size(15)
                .color(Color::from_rgb(0.3, 0.6, 0.95)),
        ),
    };

    let dismiss_btn = button(bootstrap::x_lg().size(11))
        .style(button::text)
        .on_press(Message::DismissNotification(notification.id));

    let accent_bar = container(
        iced::widget::Space::new()
            .width(ACCENT_WIDTH)
            .height(Length::Fill),
    )
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(accent_color)),
        border: Border {
            radius: Radius::new(2.0),
            ..Default::default()
        },
        ..Default::default()
    })
    .height(Length::Fill);

    let body = row![
        accent_bar,
        row![
            icon,
            text(&notification.message).size(13).width(Length::Fill),
            dismiss_btn,
        ]
        .spacing(10)
        .padding([10, 12])
        .align_y(Alignment::Center)
        .width(Length::Fill),
    ]
    .height(Length::Shrink);

    container(body)
        .width(TOAST_WIDTH)
        .style(|theme: &iced::Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(iced::Background::Color(palette.background.weak.color)),
                border: Border {
                    color: palette.background.strong.color,
                    width: 1.0,
                    radius: Radius::new(6.0),
                },
                shadow: iced::Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
                    offset: iced::Vector::new(0.0, 2.0),
                    blur_radius: 8.0,
                },
                ..Default::default()
            }
        })
        .into()
}
