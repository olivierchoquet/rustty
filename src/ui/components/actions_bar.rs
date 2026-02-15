use crate::messages::{LoginMessage, Message, ProfileMessage};
use crate::ui::theme;
use crate::ui::{theme::TerminalColors};
use iced::widget::{button, container, row, text};
use iced::{Alignment, Element, Length};

pub fn buttons_form<'a>(colors: TerminalColors, is_editing: bool) -> Element<'a, Message> {
    let texte_bouton = if is_editing { "Mettre à jour" } else { "Ajouter" };
    
    // Copies for closures - move style consumes the copy - takes exclusive ownership
    // This can be avoided with #[derive(Clone)] on TerminalColors and using .clone() 
    // here, but for simplicity we just create multiple copies
    let c1 = colors;
    let c2 = colors;
    let c3 = colors;
    let c4 = colors;

    let content = row![
        button("Nouveau")
            .on_press(Message::Profile(ProfileMessage::New))
            .style(move |_, s| theme::button_style(c1, s,theme::ButtonVariant::Secondary)), 
            
        button(text(texte_bouton).center())
            .on_press(Message::Profile(ProfileMessage::Save))
            .padding(10)
            .style(move |_, s| theme::button_style(c1, s,theme::ButtonVariant::Secondary)),

        button(text("Supprimer").center())
            .on_press(Message::Profile(ProfileMessage::Delete))
            .padding(10)
            .style(move |_, s| theme::button_style(c2, s,theme::ButtonVariant::Secondary)),

        button(text("Démarrer SSH").center())
            .on_press(Message::Login(LoginMessage::Submit))
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