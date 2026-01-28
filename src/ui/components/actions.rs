use iced::widget::{button, row, text};
use iced::{Element, Alignment};
use crate::ui::{Message, theme::TerminalColors};

pub fn buttons_form<'a>(colors: TerminalColors) -> Element<'a, Message> {
    row![
        button(text("Supprimer").center())
            .on_press(Message::DeleteProfile)
            .padding(10)
            .style(move |t, s| crate::ui::theme::button_style(colors, s)),
        
        row![
            button(text("Sauvegarder").center())
                .on_press(Message::SaveProfile)
                .padding(10)
                .style(move |t, s| crate::ui::theme::button_style(colors, s)),
                
            button(text("🚀 Démarrer SSH").center())
                .on_press(Message::ButtonConnection)
                .padding(10)
                .style(move |t, s| crate::ui::theme::active_button_style(colors, s)),
        ].spacing(10)
    ]
    .spacing(20)
    .align_y(Alignment::Center)
    .into()
}