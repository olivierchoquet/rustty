use crate::ui::theme::{self, TerminalColors, ThemeChoice};
use crate::ui::{EditSection, ID_IP, ID_PASS, ID_PORT, ID_USER, Message, MyApp, Profile};
use iced::font::Weight;
use iced::widget::{
    button, column, container, horizontal_rule, row, scrollable, text, text_input, vertical_space,
};
use iced::{Alignment, Border, Color, Element, Font, Length, Task, font};
use crate::ui::components::table::{table_content, table_header};

pub fn view(app: &MyApp) -> Element<'_, Message> {
    let colors = app.theme_choice.get_colors();

    // --- 1. SIDEBAR (LOGIQUE FIXE) ---
    let sidebar = container(
        column![
            text("NAVIGATION")
                .size(14)
                .color(colors.accent)
                .font(iced::Font {
                    weight: Weight::Bold,
                    ..Font::DEFAULT
                }),
            vertical_space().height(10),
            nav_button("Général", EditSection::General, app.active_section, colors),
            nav_button("Sécurité", EditSection::Auth, app.active_section, colors),
            nav_button("Réseum", EditSection::Network, app.active_section, colors),
            vertical_space().height(Length::Fill),
            nav_button("Avancé", EditSection::Advanced, app.active_section, colors),
            nav_button("Thèmes", EditSection::Themes, app.active_section, colors),
            button(text("Quitter").center())
                .width(Length::Fill)
                .padding(10)
                .style(move |t, s| theme::button_style(colors, s)),
        ]
        .spacing(10)
        .padding(15),
    )
    .width(Length::Fixed(200.0))
    .height(Length::Fill)
    .style(move |_| container::Style {
        background: Some(colors.surface.into()),
        ..Default::default()
    });

    // --- 2. LOGO "RustTy" (TOUJOURS VISIBLE) ---
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

    // --- 3. CONTENU DYNAMIQUE (SELON L'ONGLET) ---
    let dynamic_content: Element<_> = match app.active_section {
        EditSection::General => {
            column![
                text_input("🔍 Recherche rapide...", &app.search_query)
                    .on_input(Message::SearchChanged)
                    .padding(10)
                    .style(move |t, s| theme::input_style(colors, s)),
                // Le tableau n'apparaît que dans l'onglet Général
                column![table_header(colors), table_content(app, colors),],
                horizontal_rule(1),
                general_form(app, colors),
                action_row(colors),
            ]
            .spacing(20)
            .into()
        }

        EditSection::Auth => column![
            auth_form(app, colors),
            vertical_space().height(Length::Fill),
        ]
        .spacing(20)
        .into(),

        EditSection::Themes => column![theme_form(app, colors),].spacing(20).into(),

        _ => column![text("Section en cours de développement...").color(colors.text),]
            .spacing(20)
            .into(),
    };

    // --- 4. ASSEMBLAGE FINAL ---
    row![
        sidebar,
        container(column![brand_header, vertical_space().height(10), dynamic_content,].spacing(20))
            .padding(25)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| theme::main_container_style(colors)),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

// --- LES FONCTIONS HELPERS ---

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

fn general_form<'a>(app: &MyApp, colors: TerminalColors) -> Element<'a, Message> {
    column![
        text("Informations de base")
            .color(colors.accent)
            .font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..iced::Font::DEFAULT
            }),
        row![
            text_input("Groupe", &app.current_profile.group)
                .on_input(Message::InputNewProfileGroup)
                .padding(10)
                .width(Length::FillPortion(1))
                .style(move |t, s| theme::input_style(colors, s)),
            text_input("Nom du profil", &app.current_profile.name)
                .on_input(Message::InputNewProfileName)
                .padding(10)
                .width(Length::FillPortion(2))
                .style(move |t, s| theme::input_style(colors, s)),
        ]
        .spacing(10),
        row![
            text_input("IP ou Host", &app.current_profile.ip)
                .on_input(Message::InputIP)
                .padding(10)
                .width(Length::FillPortion(3))
                .style(move |t, s| theme::input_style(colors, s)),
            text_input("Port", &app.current_profile.port)
                .on_input(Message::InputPort)
                .padding(10)
                .width(Length::FillPortion(1))
                .style(move |t, s| theme::input_style(colors, s)),
        ]
        .spacing(10),
    ]
    .spacing(15)
    .into()
}

fn auth_form<'a>(app: &MyApp, colors: TerminalColors) -> Element<'a, Message> {
    column![
        text("Authentification SSH")
            .color(colors.accent)
            .font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..iced::Font::DEFAULT
            }),
        text_input("Utilisateur", &app.current_profile.username)
            .on_input(Message::InputUsername)
            .padding(10)
            .style(move |t, s| theme::input_style(colors, s)),
        text_input("Mot de passe", &app.password)
            .on_input(Message::InputPass)
            .secure(true) // Pour cacher les caractères
            .padding(10)
            .style(move |t, s| theme::input_style(colors, s))
            .on_submit(Message::ButtonConnection), // Entrée pour connecter
        text("Le mot de passe n'est pas stocké dans le JSON.")
            .size(12)
            .color(colors.text),
    ]
    .spacing(15)
    .into()
}

fn theme_form<'a>(app: &MyApp, colors: TerminalColors) -> Element<'a, Message> {
    let mut themes_list = column![].spacing(10);

    // On crée des lignes de 3 thèmes pour ne pas avoir une liste infinie
    let mut row_items = row![].spacing(10);

    for (i, theme) in ThemeChoice::ALL.iter().enumerate() {
        let is_selected = app.theme_choice == *theme;
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
            let mut s = theme::button_style(colors, status);
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
        scrollable(themes_list).height(Length::Fill),
    ]
    .spacing(20)
    .into()
}

pub fn update(app: &mut MyApp, message: Message) -> Task<Message> {
    match message {
        Message::InputIP(ip) => app.current_profile.ip = ip,
        Message::InputPort(port) => app.current_profile.port = port,
        Message::InputUsername(u) => app.current_profile.username = u,
        Message::InputPass(p) => app.password = p,
        Message::TabPressed => {
            // Gérer la navigation par Tab entre les champs de saisie
            app.focus_index = (app.focus_index + 1) % 4;
            let target_id = match app.focus_index {
                0 => crate::ui::ID_IP,
                1 => ID_PORT,
                2 => ID_USER,
                3 => ID_PASS,
                _ => crate::ui::ID_IP,
            };
            return text_input::focus(text_input::Id::new(target_id));
        }
        Message::ProfileSelected(profile) => {
            app.current_profile.group = profile.group.clone();
            app.current_profile.name = profile.name.clone();
            app.current_profile.ip = profile.ip.clone();
            app.current_profile.port = profile.port.clone();
            app.current_profile.username = profile.username.clone();
            app.current_profile.theme = profile.theme;
            app.selected_profile = Some(profile);
        }
        Message::InputNewProfileName(name) => {
            app.current_profile.name = name;
        }
        Message::InputNewProfileGroup(group) => {
            app.current_profile.group = group;
        }
        Message::SearchChanged(query) => {
            app.search_query = query;
        }
        Message::SaveProfile => {
            if !app.current_profile.ip.is_empty() && !app.current_profile.name.is_empty() {
                let group = if app.current_profile.group.is_empty() {
                    "DEFAUT".to_string()
                } else {
                    app.current_profile.group.to_uppercase()
                };

                let new_profile = Profile {
                    name: app.current_profile.name.clone(),
                    ip: app.current_profile.ip.clone(),
                    port: app.current_profile.port.clone(),
                    username: app.current_profile.username.clone(),
                    group,
                    theme: app.theme_choice,
                };
                app.profiles.push(new_profile);
                // On trie par groupe puis par nom pour avoir une liste propre
                app.profiles
                    .sort_by(|a, b| a.group.cmp(&b.group).then(a.name.cmp(&b.name)));
                app.save_profiles(); // On écrit sur le disque après l'ajout
            }
        }
        Message::DeleteProfile => {
            if let Some(selected) = &app.selected_profile {
                app.profiles.retain(|p| p != selected);
                app.selected_profile = None;
                // Optionnel : vider les champs après suppression
                app.current_profile.ip.clear();
                app.current_profile.username.clear();
                app.save_profiles(); // On écrit sur le disque après la suppression
            }
        }
        Message::SectionChanged(section) => {
            println!("Changement de section vers : {:?}", section);
            app.active_section = section;
            // Iced 0.13 redessine automatiquement dès que l'état change
        }
        Message::ThemeChanged(new_theme) => {
            app.theme_choice = new_theme;
            // Optionnel : println!("Thème appliqué : {:?}", new_theme);

            // Si tu as une fonction de sauvegarde, c'est le moment
            // de sauver le choix du thème pour le prochain démarrage.
            app.save_profiles();
        }
        _ => {}
    }
    Task::none()
}


fn action_row<'a>(colors: TerminalColors) -> Element<'a, Message> {
    row![
        button(text("Démarrer SSH").center())
            .on_press(Message::ButtonConnection)
            .padding(12)
            .width(150)
            .style(move |t, s| theme::button_style(colors, s)),
        button(text("Enregistrer").center())
            .on_press(Message::SaveProfile)
            .padding(12)
            .width(150)
            .style(move |t, s| theme::button_style(colors, s)),
        button(text("Supprimer").center())
            .on_press(Message::DeleteProfile)
            .padding(12)
            .width(150)
            .style(move |t, s| theme::button_style(colors, s)),
    ]
    .spacing(10)
    .into()
}
