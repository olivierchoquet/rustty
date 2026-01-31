use crate::ui::theme;
use crate::ui::{Message, MyApp, theme::TerminalColors};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Length};

pub fn header<'a>(colors: TerminalColors) -> container::Container<'a, Message> {
    container(
        row![
            text("Groupe")
                .width(Length::FillPortion(1))
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..iced::Font::DEFAULT
                }),
            text("Nom").width(Length::FillPortion(2)).font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..iced::Font::DEFAULT
            }),
            text("Utilisateur")
                .width(Length::FillPortion(1))
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..iced::Font::DEFAULT
                }),
            text("IP:Port")
                .width(Length::FillPortion(2))
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..iced::Font::DEFAULT
                }),
        ]
        .spacing(10)
        .padding(10),
    )
    .style(move |_| container::Style {
        background: Some(colors.surface.into()),
        ..Default::default()
    })
}

// On ajoute &'a avant MyApp pour dire :
// "L'app doit vivre au moins aussi longtemps que l'élément UI produit"
pub fn content<'a>(app: &'a MyApp, colors: TerminalColors) -> Element<'a, Message> {
    let mut content = column![].spacing(1);
    let query = app.search_query.to_lowercase();

    for (i, profile) in app.profiles.iter().enumerate() {
        if query.is_empty() || profile.name.to_lowercase().contains(&query) {
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
