use iced::{
    Color, Element, Font, Length, border, font::Weight, widget::{column, container, horizontal_rule, row, text, vertical_space}
};

use crate::ui::components::{actions_bar, forms::help_form, sidebar};
use crate::{
    messages::Message,
    ui::{
        EditSection, MyApp, components::{
            forms::{general_form, theme_form},
            search_table::{content, header},
        },
        theme,
    },
};

pub fn render(app: &MyApp) -> Element<'_, Message> {
    let colors = app.current_profile.theme.get_colors();

    let logs_panel = log_view(&app.logs, colors);

    let side_menu = sidebar::render(app.active_section, colors);

    // LOGO "RustTy"
    let brand_header = column![
        row![
            text("Rust").size(35).font(iced::Font {
                weight: Weight::Bold,
                ..Font::DEFAULT
            }),
            text("Ty").size(35).color(colors.accent).font(iced::Font {
                weight: Weight::Bold,
                ..Font::DEFAULT
            }),
        ],
        text("The Safety-First PuTTY Manager")
            .size(14)
            .color(Color {
                a: 0.7,
                ..colors.prompt
            })
            .font(Font {
                weight: Weight::Light,
                ..Font::DEFAULT
            }),
    ]
    .spacing(2);

    // dynamic content based on active section
    let dynamic_content: Element<_> = match app.active_section {
        EditSection::General => column![
            header(app, colors),
            content(app, colors),
            horizontal_rule(1),
            general_form(app, colors),
        ]
        .spacing(20)
        .into(),

        /*EditSection::Auth => column![
            auth_form(app, colors),
            vertical_space().height(Length::Fill),
        ]
        .spacing(20)
        .into(),*/
        EditSection::Themes => column![theme_form(app, colors),].spacing(20).into(),
        EditSection::Help => column![help_form(app,colors)].spacing(20).into(),

        _ => column![text("Section en cours de développement...").color(colors.text),]
            .spacing(20)
            .into(),
    };

    // actions bar (Save, Start SSH, ...)
    let actions_bar = actions_bar::buttons_form(colors, app.selected_profile_id.is_some());
    // FINAL ASSEMBLY
    column![
        row![
            side_menu,
            container(
                column![
                    brand_header,
                    vertical_space().height(10),
                    container(dynamic_content)
                        .height(Length::Fill) 
                        .width(Length::Fill),
                    logs_panel,
                ]
                .spacing(10)
            )
            .padding(10)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| theme::main_container_style(colors)),
        ]
        .width(Length::Fill)
        .height(Length::Fill),
        actions_bar,
    ]
    .align_x(iced::alignment::Horizontal::Center)
    .into()
}

pub fn log_view<'a>(
    logs: &'a [String],
    colors: crate::ui::theme::TerminalColors,
) -> Element<'a, Message> {
    let log_content = column(
        logs.iter()
            .take(15) // Take last recents logs
            .map(|l| {
                let log_color = if l.contains("ERROR") || l.contains("Échec") {
                    Color::from_rgb(1.0, 0.4, 0.4) // ERROR always red
                } else if l.contains("INFO") {
                    colors.accent 
                } else {
                    colors.text
                };
                text(l)
                    .size(14)
                    .font(Font::MONOSPACE)
                    .color(log_color) // On garde le blanc pour le debug
                    .into()
            })
            .collect::<Vec<Element<'a, Message>>>(),
    )
    .spacing(2);

    container(log_content)
        .width(Length::Fill)
        .height(Length::Fixed(200.0)) // Fixe pour être sûr qu'il ne disparaisse pas
        .padding(10)
        .style(move |_| container::Style {
            background: Some(Color {
                    a: 0.05,
                    ..colors.accent
            }.into()),
            border: border::rounded(5)
                .width(1.0)
                .color(colors.accent),
                
            ..Default::default()
        })
        .into()
}
