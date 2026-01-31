use iced::Element;
use iced::widget::{column, horizontal_rule};
use crate::ui::components::actions;
use crate::ui::{Message, MyApp, theme::TerminalColors, components::{table, forms}};

pub fn render<'a>(app: &'a MyApp, colors: TerminalColors) -> Element<'a, Message> {
    column![
        // Partie haute : Le tableau des profils
        column![
            table::header(app,colors),
            table::content(app, colors),
        ],
        
        horizontal_rule(1),

        // Partie basse : Édition du profil sélectionné
        forms::general_form(app, colors),
        actions::buttons_form(colors, app.selected_profile_id.is_some()),
    ]
    .spacing(20)
    .into()
}