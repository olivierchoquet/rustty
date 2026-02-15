use crate::messages::{ConfigMessage, LoginMessage, Message, ProfileMessage};
use crate::ui::theme::{self, ThemeChoice};
use crate::ui::{MyApp, theme::TerminalColors};
use iced::alignment::{Horizontal, Vertical};
use iced::font::Weight;
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Color, Element, Font, Length};

// general form (sidebar)
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
            render_input_with_label(
                "Nom du profil",
                &app.current_profile.name,
                text_input::Id::new("profile_input"),
                colors,
                None,
                false,
                |s| Message::Profile(ProfileMessage::InputName(s)),
                None,
            ),
            render_input_with_label(
                "Groupe",
                &app.current_profile.group,
                text_input::Id::new("group_input"),
                colors,
                None,
                false,
                |s| Message::Profile(ProfileMessage::InputGroup(s)),
                None
            ),
        ]
        .spacing(10),
        row![
            render_input_with_label(
                "Adresse IP",
                &app.current_profile.ip,
                text_input::Id::new("ip_input"),
                colors,
                None,
                false,
                |s| Message::Login(LoginMessage::InputIP(s)),
                None
            ),
            render_input_with_label(
                "Port",
                &app.current_profile.port,
                text_input::Id::new("port_input"),
                colors,
                None,
                false,
                |s| Message::Login(LoginMessage::InputPort(s)),
                None
            ),
        ]
        .spacing(10),
        row![
            render_input_with_label(
                "Nom d'utilisateur",
                &app.current_profile.username,
                text_input::Id::new("user_input"),
                colors,
                None,
                false,
                |s| Message::Login(LoginMessage::InputUsername(s)),
                None,
            ),
            render_input_with_label(
                "Mot de passe",
                &app.password,
                text_input::Id::new("pass_input"),
                colors,
                Some("⚠️ Non enregistré dans le profil pour votre sécurité"),
                true,
                |s| Message::Login(LoginMessage::InputPass(s)),
                Some(Message::Login(LoginMessage::Submit))
            ),
        ]
        .spacing(10),
        column![
            text("OPTIONS DE SESSION").size(12).color(colors.accent),
            terminal_count_selector(app.current_profile.terminal_count, colors),
        ]
        .spacing(8),
    ]
    .spacing(15)
    .into()
}

fn render_input_with_label<'a>(
    label: &'a str,
    value: &'a str,
    id: text_input::Id, // ID for focus management
    colors: TerminalColors,
    helper_text: Option<&'a str>,
    is_secure: bool,
    msg: impl Fn(String) -> Message + 'a,
    on_submit_message: Option<Message>,
) -> Element<'a, Message> {
    let mut col = column![
        // label
        text(label).size(13).style(move |_| text::Style {
            color: Some(colors.text.into())
        }),
        // input
        text_input(label, value)
            .id(id)
            .on_input(msg)
            .padding(10)
            .secure(is_secure)
            .on_submit(on_submit_message.unwrap_or(Message::DoNothing)),
    ]
    .spacing(5)
    .width(Length::Fill)
    .height(Length::Shrink);

    // optional text helper below the input (e.g., for password warning)
    if let Some(help) = helper_text {
        col = col.push(text(help).size(11).style(move |_| text::Style {
            color: Some(colors.prompt.into()),
        }));
    }

    col.into()
}

pub fn terminal_count_selector<'a>(
    current_count: usize,
    colors: TerminalColors,
) -> Element<'a, Message> {
    row![
        text("Nombre de fenêtres :")
            .width(Length::Fill)
            .color(colors.text),
        button(
            text("-")
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
        )
        .on_press(Message::Profile(ProfileMessage::TerminalCountChanged(
            current_count.saturating_sub(1).max(1)
        )))
        .width(35),

        container(text(current_count.to_string()).color(colors.text).size(18))
            .width(40)
            .align_x(Horizontal::Center),

        button(
            text("+")
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
        )
        .on_press(Message::Profile(ProfileMessage::TerminalCountChanged(
            current_count.saturating_add(1).min(4) // max 4 windows
        )))
        .width(35),
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .into()
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
                    // Preview of the theme colors
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
        .on_press(Message::Config(ConfigMessage::ThemeChanged(*theme)))
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
