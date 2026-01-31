use crate::ui::theme;
use crate::ui::{Message, theme::TerminalColors};
use iced::widget::{button, row, text};
use iced::{Alignment, Element};

pub fn buttons_form<'a>(colors: TerminalColors, is_editing: bool) -> Element<'a, Message> {
    let texte_bouton = if is_editing { "Mettre à jour" } else { "Ajouter" };
    row![
        button("Nouveau")
            .on_press(Message::NewProfile)
            .style(button::secondary), // Style différent (gris par défaut)
        button(text(texte_bouton).center())
            .on_press(Message::SaveProfile)
            .padding(10)
            .style(move |t, s| theme::button_style(colors, s)),
        button(text("Supprimer").center())
            .on_press(Message::DeleteProfile)
            .padding(10)
            .style(move |t, s| theme::button_style(colors, s)),
        button(text("Démarrer SSH").center())
            .on_press(Message::ButtonConnection)
            .padding(10)
            .style(move |t, s| crate::ui::theme::active_button_style(colors, s)),
        button(text("Quitter").center())
            .padding(10)
            .on_press(Message::QuitRequested)
            .style(move |t, s| theme::button_style(colors, s))
    ]
    .spacing(20)
    .padding([15, 0]) // Ajoute 15px de marge interne en BAS
    .align_y(Alignment::Center)
    .into()
}
