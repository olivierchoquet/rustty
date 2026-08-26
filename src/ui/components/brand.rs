use crate::messages::Message;
use iced::{Element, widget::image};

pub fn logo() -> Element<'static, Message> {
    image("assets/rustty.png").into()
}
