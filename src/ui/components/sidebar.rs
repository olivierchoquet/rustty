use iced::widget::{button, column, container, text, vertical_space};
use iced::{Alignment, Border, Element, Length};
use crate::ui::{Message, EditSection, theme::{self, TerminalColors}};

pub fn render<'a>(active_section: EditSection, colors: TerminalColors) -> Element<'a, Message> {
    container(
        column![
            text("NAVIGATION")
                .size(14)
                .color(colors.accent)
                .font(iced::Font { weight: iced::font::Weight::Bold, ..iced::Font::DEFAULT }),
            
            vertical_space().height(10),
            
            nav_button("Général", EditSection::General, active_section, colors),
            nav_button("Sécurité", EditSection::Auth, active_section, colors),
            nav_button("Réseau", EditSection::Network, active_section, colors),
            
            vertical_space().height(Length::Fill),
            
            nav_button("Avancé", EditSection::Advanced, active_section, colors),
            nav_button("Thèmes", EditSection::Themes, active_section, colors),
            
            button(text("Quitter").center())
                .width(Length::Fill)
                .padding(10)
                .on_press(Message::QuitRequested)
                .style(move |t, s| theme::button_style(colors, s)),
        ]
        .spacing(10)
        .padding(15)
    )
    .width(Length::Fixed(200.0))
    .height(Length::Fill)
    .style(move |_| container::Style {
        background: Some(colors.surface.into()),
        ..Default::default()
    })
    .into()
}

fn nav_button<'a>(
    label: &'a str,
    section: EditSection,
    active: EditSection,
    colors: TerminalColors,
) -> button::Button<'a, Message> {
    let is_active = section == active;

    button(text(label).width(Length::Fill).center())
        .on_press(Message::SectionChanged(section))
        .padding(10)
        .style(move |_, status| {
            // 1. On récupère le style de base
            let mut s = theme::button_style(colors, status);

            // 2. ON ÉCRASE SYSTÉMATIQUEMENT SI ACTIF
            if is_active {
                s.background = Some(colors.accent.into());
                s.text_color = iced::Color::BLACK;
                s.border = Border {
                    color: colors.accent,
                    width: 1.0,
                    radius: 5.0.into(),
                };
            } else {
                // Optionnel: on peut aussi forcer le style inactif ici
                s.background = Some(colors.surface.into());
                s.text_color = iced::Color::WHITE;
            }
            s
        })
}