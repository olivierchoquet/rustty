use iced::widget::image;
use iced::{Element, Length};
use crate::ui::Message;

pub fn logo() -> Element<'static, Message> {
    // On charge l'image à partir du chemin relatif
    image("assets/rustty.png")
        //.width(Length::Fixed(150.0)) // Tu peux ajuster la taille ici
        .into()
}