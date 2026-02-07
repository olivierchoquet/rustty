use crate::ui::theme;
use crate::ui::{Message, theme::TerminalColors};
use iced::widget::{button, container, row, text};
use iced::{Alignment, Element, Length};

pub fn buttons_form<'a>(colors: TerminalColors, is_editing: bool) -> Element<'a, Message> {
    let texte_bouton = if is_editing { "Mettre à jour" } else { "Ajouter" };
    
    // Copies pour les closures - style move consomme la copie - prend la propriété exclusive
    // On peut éviter ceci  par #
    let c1 = colors;
    let c2 = colors;
    let c3 = colors;
    let c4 = colors;

    let content = row![
        button("Nouveau")
            .on_press(Message::NewProfile)
            .style(move |_, s| theme::button_style(c1, s,theme::ButtonVariant::Secondary)), 
            
        button(text(texte_bouton).center())
            .on_press(Message::SaveProfile)
            .padding(10)
            .style(move |_, s| theme::button_style(c1, s,theme::ButtonVariant::Secondary)),

        button(text("Supprimer").center())
            .on_press(Message::DeleteProfile)
            .padding(10)
            .style(move |_, s| theme::button_style(c2, s,theme::ButtonVariant::Secondary)),

        button(text("Démarrer SSH").center())
            .on_press(Message::ButtonConnection)
            .padding(10)
            .style(move |_, s| theme::button_style(c3, s,theme::ButtonVariant::Primary)),

        button(text("Quitter").center())
            .padding(10)
            .on_press(Message::QuitRequested)
            .style(move |_, s| theme::button_style(c4, s,theme::ButtonVariant::Secondary))
    ]
    .spacing(20)
    .align_y(Alignment::Center);
    container(content)
        .width(Length::Fill)
        .center_x(Length::Fill) // <--- Ajoute ceci pour le centrage horizontal
        .padding(15)
        .style(move |_| container::Style {
            background: Some(colors.bg.into()),
            ..Default::default()
        })
        .into()
}