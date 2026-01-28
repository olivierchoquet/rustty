use iced::Element;
use iced::widget::column;
use crate::ui::{Message, MyApp, theme::TerminalColors, components::forms};

pub fn render<'a>(app: &'a MyApp, colors: TerminalColors) -> Element<'a, Message> {
    column![
        // On affiche le sélecteur de thèmes
        forms::themes::render(app, colors),
        
        // Pas forcément besoin de "Démarrer SSH" ici, 
        // mais on garde "Sauvegarder" via actions pour la cohérence
        forms::actions::render(colors),
    ]
    .spacing(20)
    .into()
}