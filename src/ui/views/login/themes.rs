use iced::Element;
use iced::widget::column;
use crate::{messages::Message, ui::{MyApp, components::{actions, forms}, theme::TerminalColors}};

pub fn render<'a>(app: &'a MyApp, colors: TerminalColors) -> Element<'a, Message> {
    column![
        forms::theme_form(app, colors),
        actions::buttons_form(colors, app.selected_profile_id.is_some()),
    ]
    .spacing(20)
    .into()
}