use iced::{Element, widget::image};
use crate::messages::Message;

pub fn logo() -> Element<'static, Message> {
    image("assets/rustty.png")
        .into()
}