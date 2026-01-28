use iced::Element;
use iced::widget::{column, horizontal_rule};
use crate::ui::{Message, MyApp, theme::TerminalColors, components::{table, forms}};

pub fn render<'a>(app: &'a MyApp, colors: TerminalColors) -> Element<'a, Message> {
    column![
        // Partie haute : Le tableau des profils
        column![
            table::header(colors),
            table::content(app, colors),
        ],
        
        horizontal_rule(1),

        // Partie basse : Édition du profil sélectionné
        forms::general::render(app, colors),
        forms::actions::render(colors),
    ]
    .spacing(20)
    .into()
}