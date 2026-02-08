use iced::Element;
use iced::widget::column;
use crate::{messages::Message, ui::{MyApp, components::{actions, forms}, theme::TerminalColors}};

pub fn render<'a>(app: &'a MyApp, colors: TerminalColors) -> Element<'a, Message> {
    column![
        // On affiche le sélecteur de thèmes
        forms::theme_form(app, colors),
        
        // Pas forcément besoin de "Démarrer SSH" ici, 
        // mais on garde "Sauvegarder" via actions pour la cohérence
        actions::buttons_form(colors, app.selected_profile_id.is_some()),
    ]
    .spacing(20)
    .into()
}