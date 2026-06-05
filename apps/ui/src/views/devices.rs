use iced::widget::{button, column, container, text};
use iced::{Element, Length};

use crate::app::App;
use crate::message::Message;

pub fn view(app: &App) -> Element<'_, Message> {
    let translations = app.translations();

    let detect_btn = button(text(translations.detect_btn).size(14))
        .on_press(Message::DetectDevicesPressed);

    let body: Element<Message> = if app.detecting_devices {
        text(translations.detecting).size(14).into()
    } else if app.devices.is_empty() {
        text(translations.devices_hint).size(14).into()
    } else {
        let rows: Vec<Element<Message>> = app
            .devices
            .iter()
            .map(|device| {
                let path = device.path.clone();
                button(text(device.to_string()).size(13))
                    .width(Length::Fill)
                    .style(button::secondary)
                    .on_press(Message::DeviceSelected(path))
                    .into()
            })
            .collect();
        column(rows).spacing(4).into()
    };

    container(column![detect_btn, body].spacing(16))
        .padding(24)
        .width(Length::Fill)
        .into()
}
