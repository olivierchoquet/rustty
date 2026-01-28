use iced::widget::{column, container, row, vertical_space};
use iced::{Element, Length};
use crate::ui::{Message, MyApp, theme, components::sidebar};
use super::login; // Accès aux fichiers frères (general, auth, etc.)

// Déclaration des sous-modules (les fichiers general.rs, auth.rs, themes.rs)
pub mod general;
pub mod auth;
pub mod themes;
pub mod views;

// Ré-exportation de la fonction layout principale
pub mod layout; 
pub use layout::render;

pub fn render(app: &MyApp) -> Element<'_, Message> {
    let colors = app.theme_choice.get_colors();

    // 1. On récupère la sidebar (composant)
    let side_menu = sidebar::render(app.active_section, colors);

    // 2. On définit l'en-tête (Logo)
    let brand_header = column![
        row![
            iced::widget::text("Rust").size(35).bold(),
            iced::widget::text("Ty").size(35).color(colors.accent).bold(),
        ],
        iced::widget::text("The Safety-First PuTTY Manager").size(14).color(colors.prompt),
    ].spacing(2);

    // 3. Aiguillage vers l'onglet actif (C'est ici qu'on appelle les autres fichiers du dossier)
    let tab_content = match app.active_section {
        crate::ui::EditSection::General => login::general::render(app, colors),
        crate::ui::EditSection::Auth    => login::auth::render(app, colors),
        crate::ui::EditSection::Themes  => login::themes::render(app, colors),
        _ => iced::widget::text("Onglet en cours de développement...").into(),
    };

    // 4. Assemblage final
    row![
        side_menu,
        container(
            column![
                brand_header,
                vertical_space().height(10),
                tab_content,
            ]
            .spacing(20)
        )
        .padding(25)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| theme::main_container_style(colors))
    ].into()
}