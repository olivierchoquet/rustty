use iced::{Element, Task, widget::{text_input, column, row, text, container, vertical_space, horizontal_rule}, Font, Color, Length, font::Weight};

use crate::ui::{EditSection, ID_PASS, ID_PORT, ID_USER, Message, MyApp, Profile, components::{actions, forms::{auth_form, general_form, theme_form}, sidebar, table::{content, header}}, theme};

// On dit juste à Rust que les fichiers à côté existent
pub mod general;
pub mod auth;
pub mod themes;

pub fn render(app: &MyApp) -> Element<'_, Message> {
    let colors = app.theme_choice.get_colors();

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
                text_input("🔍 Recherche rapide...", &app.search_query)
                    .on_input(Message::SearchChanged)
                    .padding(10)
                    .style(move |t, s| theme::input_style(colors, s)),
                // Le tableau n'apparaît que dans l'onglet Général
                column![header(colors), content(app, colors),],
                horizontal_rule(1),
                general_form(app, colors),
                actions::buttons_form(colors),
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
        side_menu,
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