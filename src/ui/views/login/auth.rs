use iced::Element;
use iced::widget::column;
use crate::ui::{Message, MyApp, theme::TerminalColors, components::forms};

pub fn render<'a>(app: &'a MyApp, colors: TerminalColors) -> Element<'a, Message> {
    column![
        // On affiche le formulaire SSH/Auth que nous avons créé dans components
        forms::auth::render(app, colors),
        
        // On ajoute la barre d'actions (Sauvegarder/Démarrer) en bas
        forms::actions::render(colors),
    ]
    .spacing(20)
    .into()
}