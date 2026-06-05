use iced::widget::{button, column, container, text};
use iced::{Element, Length};

use crate::app::App;
use crate::message::Message;

pub fn view(app: &App) -> Element<'_, Message> {
    let detect_btn = button(text("Detectar Dispositivos").size(14))
        .on_press(Message::DetectDevicesPressed);

    let body: Element<Message> = if app.detecting_devices {
        text("Detectando dispositivos…").size(14).into()
    } else if app.devices.is_empty() {
        text("Clique em 'Detectar Dispositivos' para listar os dispositivos disponíveis.")
            .size(14)
            .into()
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
