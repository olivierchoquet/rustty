use iced::{
    Alignment, Color, Element, Font, Length, Task,
    font::Weight,
    widget::{column, container, horizontal_rule, row, text, text_input, vertical_space},
};

use crate::{messages::{ConfigMessage, LoginMessage, Message, ProfileMessage}, ui::{EditSection, MyApp, Profile, components::{forms::{general_form, theme_form}, search_table::{content, header}}, theme}};
use crate::ui::components::{actions_bar, sidebar};

//use crate::ui::constants::*;


pub fn render(app: &MyApp) -> Element<'_, Message> {
    let colors = app.current_profile.theme.get_colors();

    // 1. Appel du composant Sidebar
    let side_menu = sidebar::render(app.active_section, colors);

    // 2. LOGO "RustTy"
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
            }) // On baisse l'opacité pour donner un style "secondaire"
            .font(Font {
                weight: Weight::Light, // Ou Weight::Normal si Light n'est pas dispo
                ..Font::DEFAULT
            }),
    ]
    .spacing(2);

    // 3. CONTENU DYNAMIQUE (SELON L'ONGLET) ---
    let dynamic_content: Element<_> = match app.active_section {
        EditSection::General => {
            column![
                // L'onglet Général appelle le header (qui contient maintenant la recherche)
                header(app, colors),
                // Le contenu du tableau (les lignes)
                content(app, colors),
                horizontal_rule(1),
                // Le formulaire
                general_form(app, colors),
            ]
            .spacing(20)
            .into()
        }

        /*EditSection::Auth => column![
            auth_form(app, colors),
            vertical_space().height(Length::Fill),
        ]
        .spacing(20)
        .into(),*/
        EditSection::Themes => column![theme_form(app, colors),].spacing(20).into(),

        _ => column![text("Section en cours de développement...").color(colors.text),]
            .spacing(20)
            .into(),
    };

    // 4. Barre d'actions commune à tous les onglets (Sauvegarder, Supprimer, Démarrer, Quitter)
    let actions_bar = actions_bar::buttons_form(colors, app.selected_profile_id.is_some());
    // 5. ASSEMBLAGE FINAL ---
    column![
        row![
            side_menu,
            container(
                column![brand_header, vertical_space().height(10), dynamic_content].spacing(20)
            )
            .padding(25)
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


