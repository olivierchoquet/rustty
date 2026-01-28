use iced::Element;
use iced::widget::column;
use crate::ui::{Message, MyApp, components::{actions, forms}, theme::TerminalColors};

pub fn render<'a>(app: &'a MyApp, colors: TerminalColors) -> Element<'a, Message> {
    column![
        // On affiche le formulaire SSH/Auth que nous avons créé dans components
        forms::auth_form(app, colors),
        
        // On ajoute la barre d'actions (Sauvegarder/Démarrer) en bas
        actions::buttons_form(colors),
    ]
    .spacing(20)
    .into()
}

