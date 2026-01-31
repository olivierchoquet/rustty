use crate::ui::theme;
use crate::ui::{Message, MyApp, theme::TerminalColors};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Font, Length, font};

pub fn header<'a>(app: &'a MyApp, colors: TerminalColors) -> Element<'a, Message> {
    column![
        // Barre de recherche
        text_input("🔍 Recherche rapide sur nom, groupe, ip, utilisateur", &app.search_query)
            .on_input(Message::SearchChanged)
            .padding(10)
            .style(move |theme: &iced::Theme, status| {
                theme::input_style(colors, status)
            }),

        // --- ESPACE ICI ---
        
        // Ligne des titres
        container(
            row![
                bold_text("GROUPE").width(Length::FillPortion(1)),
                bold_text("NOM").width(Length::FillPortion(2)),
                bold_text("UTILISATEUR").width(Length::FillPortion(1)),
                bold_text("ADRESSE IP").width(Length::FillPortion(2)),
            ]
            .spacing(10)
        )
        .padding(10)
        .style(move |_theme| {
            container::Style {
                background: Some(colors.bg.into()),
                text_color: Some(colors.text.into()),
                ..Default::default()
            }
        })
    ]
    .spacing(20) // <--- Augmente cette valeur pour plus d'espace sous la recherche
    .into()
}

// On ajoute &'a avant MyApp pour dire :
// "L'app doit vivre au moins aussi longtemps que l'élément UI produit"
pub fn content<'a>(app: &'a MyApp, colors: TerminalColors) -> Element<'a, Message> {
    let mut content = column![].spacing(1);
    let query = app.search_query.to_lowercase();

    for (i, profile) in app.profiles.iter().enumerate() {
        let is_match = query.is_empty() 
            || profile.name.to_lowercase().contains(&query) 
            || profile.group.to_lowercase().contains(&query) // Recherche par Groupe
            || profile.ip.contains(&query)          // Recherche par IP
            || profile.username.to_lowercase().contains(&query); // Recherche par Nom d'utilisateur
        if is_match {
            let is_selected = app.selected_profile_id == Some(profile.id);
            let zebra_color = if i % 2 == 0 {
                colors.surface
            } else {
                colors.bg
            };

            // On utilise .push(...) normalement
            content = content.push(
                button(
                    container(
                        row![
                            text(&profile.group).width(Length::FillPortion(1)),
                            text(&profile.name).width(Length::FillPortion(2)),
                            text(&profile.username).width(Length::FillPortion(1)),
                            text(format!("{}:{}", profile.ip, profile.port))
                                .width(Length::FillPortion(2)),
                        ]
                        .spacing(10),
                    )
                    .padding(8),
                )
                .width(Length::Fill)
                .on_press(Message::ProfileSelected(profile.id))
                .style(move |_, status| {
                    let mut st = theme::button_style(colors, status);
                    if is_selected {
                        st.background = Some(colors.prompt.into());
                        st.text_color = colors.accent;
                        // AJOUT : Une bordure pour bien marquer le coup
                        st.border.width = 2.0;
                        st.border.color = colors.accent;
                    } else {
                        // MAJ : Couleur zébrée pour les lignes non sélectionnées
                        st.background = Some(zebra_color.into());
                        st.text_color = colors.text;
                        st.border.width = 0.0;
                    }
                    st
                }),
            );
        }
    }

    // Maintenant .into() fonctionnera car les durées de vie sont liées
    scrollable(content).height(Length::Fixed(150.0)).into()
}


// Fonction utilitaire pour créer un texte en gras
fn bold_text(content: &str) -> text::Text {
    text(content).font(Font {
        weight: font::Weight::Bold,
        ..Default::default()
    })
}