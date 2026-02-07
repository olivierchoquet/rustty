use crate::ui::theme::{self, ThemeChoice};
use crate::ui::{Message, MyApp, theme::TerminalColors};
use iced::font::Weight;
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Color, Element, Font, Length, Theme};

// Le formulaire pour l'onglet Général
pub fn general_form<'a>(app: &'a MyApp, colors: TerminalColors) -> Element<'a, Message> {
    column![
        text("ÉDITION DU PROFIL")
            .size(14)
            .font(Font {
                weight: Weight::Bold,
                ..Font::default()
            })
            .color(colors.accent),
        row![
            /*text_input("Nom du profil", &app.current_profile.name)
            .on_input(Message::InputNewProfileName)
            .padding(10)
            .style(move |t, s| crate::ui::theme::input_style(colors, s)),*/
            render_input_with_label(
                "Nom du profil",
                &app.current_profile.name,
                colors,
                None,
                false,
                Message::InputNewProfileName,
                None,
            ),
            render_input_with_label(
                "Groupe",
                &app.current_profile.group,
                colors,
                None,
                false,
                Message::InputNewProfileGroup,
                None
            ),
        ]
        .spacing(10),
        row![
            render_input_with_label(
                "Adresse IP",
                &app.current_profile.ip,
                colors,
                None,
                false,
                Message::InputIP,
                None
            ),
            render_input_with_label(
                "Port",
                &app.current_profile.port,
                colors,
                None,
                false,
                Message::InputPort,
                None
            ),
        ]
        .spacing(10),
        row![
            render_input_with_label(
                "Nom d'utilisateur",
                &app.current_profile.username,
                colors,
                None,
                false,
                Message::InputUsername,
                None,
            ),
            render_input_with_label(
                "Mot de passe",
                &app.password,
                colors,
                Some("⚠️ Non enregistré dans le profil pour votre sécurité"),
                true,
                Message::InputPass,
                Some(Message::ButtonConnection)
            ),
        ]
        .spacing(10),
    ]
    .spacing(15)
    .into()
}

fn render_input_with_label<'a>(
    label: &'a str,
    value: &'a str,
    colors: TerminalColors, // On va s'en servir !
    helper_text: Option<&'a str>,
    is_secure: bool,
    msg: impl Fn(String) -> Message + 'a,
    on_submit_message: Option<Message>,
) -> Element<'a, Message> {
    let mut col = column![
        // 1. Label utilisant la couleur de texte de ton thème (atténuée)
        text(label).size(13).style(move |_| text::Style {
            // On utilise la couleur d'accent ou de texte de ton struct
            color: Some(colors.text.into())
        }),
        // 2. Input
        text_input(label, value)
            .on_input(msg)
            .padding(10)
            .secure(is_secure)
            .on_submit(on_submit_message.unwrap_or(Message::DoNothing)),
    ]
    .spacing(5)
    .width(Length::Fill) // Prend toute la largeur dispo
    .height(Length::Shrink); // Ne prend QUE la hauteur nécessaire

    // 3. Helper Text utilisant une couleur d'alerte du thème
    if let Some(help) = helper_text {
        col = col.push(text(help).size(11).style(move |_| text::Style {
            color: Some(colors.prompt.into()), // On utilise 'prompt' pour l'alerte
        }));
    }

    col.into()
}

// Le formulaire pour l'onglet Auth  - NON UTILISE POUR L'INSTANT - utilisateur et mot de passe dans l'onglet Général
/* pub fn auth_form<'a>(app: &'a MyApp, colors: TerminalColors) -> Element<'a, Message> {
    column![
        text("SÉCURITÉ ET ACCÈS")
            .size(14)
            .font(Font {
                weight: Weight::Bold,
                ..Font::default()
            })
            .color(colors.accent),
        text_input("Nom d'utilisateur", &app.current_profile.username)
            .on_input(Message::InputUsername)
            .padding(10)
            .style(move |t, s| crate::ui::theme::input_style(colors, s)),
        text_input("Mot de passe", &app.password)
            .on_input(Message::InputPass)
            .secure(true)
            .padding(10)
            .style(move |t, s| crate::ui::theme::input_style(colors, s)),
    ]
    .spacing(15)
    .into()
}
*/

pub fn theme_form<'a>(app: &MyApp, colors: TerminalColors) -> Element<'a, Message> {
    let mut themes_list = column![].spacing(10);

    // On crée des lignes de 3 thèmes pour ne pas avoir une liste infinie
    let mut row_items = row![].spacing(10);

    for (i, theme) in ThemeChoice::ALL.iter().enumerate() {
        let is_selected = app.current_profile.theme == *theme;
        let theme_colors = theme.get_colors();

        let theme_button = button(
            container(
                column![
                    text(format!("{}", theme)).size(14).font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..iced::Font::DEFAULT
                    }),
                    // Petite prévisualisation des couleurs du thème
                    row![
                        container(text("")).width(15).height(15).style(move |_| {
                            container::Style {
                                background: Some(theme_colors.accent.into()),
                                ..Default::default()
                            }
                        }),
                        container(text("")).width(15).height(15).style(move |_| {
                            container::Style {
                                background: Some(theme_colors.prompt.into()),
                                ..Default::default()
                            }
                        }),
                        container(text("")).width(15).height(15).style(move |_| {
                            container::Style {
                                background: Some(theme_colors.bg.into()),
                                ..Default::default()
                            }
                        }),
                    ]
                    .spacing(5)
                ]
                .spacing(5),
            )
            .padding(10)
            .width(Length::Fill)
            .center_x(Length::Fill),
        )
        .on_press(Message::ThemeChanged(*theme))
        .style(move |_, status| {
            let mut s = theme::button_style(colors, status, theme::ButtonVariant::Secondary);
            if is_selected {
                s.border.width = 2.0;
                s.border.color = colors.accent;
                s.background = Some(colors.surface.into());
                s.text_color = colors.accent;
            } else {
                s.background = Some(colors.surface.into());
                s.text_color = colors.text;
                s.border.width = 1.0;
                s.border.color = Color::from_rgba(1.0, 1.0, 1.0, 0.1);
            }
            s
        });

        row_items = row_items.push(theme_button);

        // Toutes les 3 vignettes, on crée une nouvelle ligne
        if (i + 1) % 3 == 0 || (i + 1) == ThemeChoice::ALL.len() {
            themes_list = themes_list.push(row_items);
            row_items = row![].spacing(10);
        }
    }

    column![
        text("Personnalisation de l'interface")
            .size(20)
            .color(colors.accent)
            .font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..iced::Font::DEFAULT
            }),
        text("Choisissez un thème visuel pour votre terminal et l'application.")
            .size(14)
            .color(colors.text),
        text("Vous pouvez choisir un thème différent pour chaque profil.")
            .size(14)
            .color(colors.text),
        scrollable(themes_list).height(Length::Fill),
    ]
    .spacing(20)
    .into()
}
