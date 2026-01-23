use crate::ui::theme::{self};
use crate::ui::{ID_IP, ID_PASS, ID_PORT, ID_USER, Message, MyApp, Session};
use iced::font::Weight;
use iced::widget::{button, column, container, horizontal_rule, row, scrollable, text, text_input};
use iced::{Alignment, Border, Element, Length, Task, font};

pub fn view(app: &MyApp) -> Element<'_, Message> {
    let colors = app.theme_choice.get_colors();

    // --- 1. EN-TÊTE DU DATAGRID ---
    let table_header = container(
        row![
            text("Groupe")
                .width(Length::FillPortion(1))
                .font(font::Font {
                    weight: Weight::Bold,
                    ..Default::default()
                }),
            text("Nom").width(Length::FillPortion(2)).font(font::Font {
                weight: Weight::Bold,
                ..Default::default()
            }),
            text("Utilisateur")
                .width(Length::FillPortion(1))
                .font(font::Font {
                    weight: Weight::Bold,
                    ..Default::default()
                }),
            text("IP:Port")
                .width(Length::FillPortion(2))
                .font(font::Font {
                    weight: Weight::Bold,
                    ..Default::default()
                }),
        ]
        .spacing(10)
        .padding(10),
    )
    .style(move |_| container::Style {
        background: Some(colors.surface.into()),
        ..Default::default()
    });

    // --- 2. CONTENU DU DATAGRID (LIGNES) ---
    let mut table_content = column![].spacing(1);
    let query = app.search_query.to_lowercase();

    let filtered_sessions = app.sessions.iter().filter(|s| {
        s.group.to_lowercase().contains(&query)
            || s.name.to_lowercase().contains(&query)
            || s.ip.contains(&query)
            || s.username.to_lowercase().contains(&query)
    });

    for (i, session) in filtered_sessions.enumerate() {
        let is_selected = app.selected_session.as_ref() == Some(session);

        // On alterne entre 'surface' et 'bg' (le fond de la fenêtre)
        let zebra_color = if i % 2 == 0 {
            colors.surface
        } else {
            colors.bg // <--- Assure-toi que ce champ existe dans TerminalColors
        };

        let row_item = button(
            container(
                row![
                    text(&session.group).width(Length::FillPortion(1)).size(14),
                    text(&session.name).width(Length::FillPortion(2)).size(14),
                    text(&session.username)
                        .width(Length::FillPortion(1))
                        .size(14),
                    text(format!("{}:{}", session.ip, session.port))
                        .width(Length::FillPortion(2))
                        .size(14),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            )
            .padding(8),
        )
        .width(Length::Fill)
        .on_press(Message::SessionSelected(session.clone()))
        .style(move |_theme, status| {
            let mut style = theme::button_style(colors, status);
            if is_selected {
                // .into() transforme la Color en Background (obligatoire en 0.13)
                style.background = Some(colors.prompt.into());
                style.text_color = colors.accent;
            } else {
                // .into() ici aussi pour la couleur zebra
                style.background = Some(zebra_color.into());
                style.text_color = colors.text;
            }
            style.border = Border {
                color: colors.surface,
                width: 1.0,
                radius: 0.0.into(),
            };
            style
        });

        table_content = table_content.push(row_item);
    }

    // --- 3. FORMULAIRE D'ENCODAGE ---
    let encoding_form = column![
        text("Édition de la session")
            .size(18)
            .color(colors.accent)
            .font(font::Font {
                weight: Weight::Bold,
                ..Default::default()
            }),
        row![
            text_input("Groupe (ex: PROD)", &app.current_session.group)
                .on_input(Message::InputNewSessionGroup)
                .padding(10)
                .width(Length::FillPortion(1))
                .style(move |t, s| theme::input_style(colors, s)),
            text_input("Nom du serveur", &app.current_session.name)
                .on_input(Message::InputNewSessionName)
                .padding(10)
                .width(Length::FillPortion(2))
                .style(move |t, s| theme::input_style(colors, s)),
        ]
        .spacing(10),
        row![
            text_input("IP", &app.current_session.ip)
                .id(text_input::Id::new(ID_IP))
                .on_input(Message::InputIP)
                .padding(10)
                .width(Length::FillPortion(3))
                .style(move |t, s| theme::input_style(colors, s)),
            text_input("Port", &app.current_session.port)
                .id(text_input::Id::new(ID_PORT))
                .on_input(Message::InputPort)
                .padding(10)
                .width(Length::FillPortion(1))
                .style(move |t, s| theme::input_style(colors, s)),
        ]
        .spacing(10),
        row![
            text_input("Utilisateur", &app.current_session.username)
                .id(text_input::Id::new(ID_USER))
                .on_input(Message::InputUsername)
                .padding(10)
                .width(Length::Fill)
                .style(move |t, s| theme::input_style(colors, s)),
            text_input("Mot de passe", &app.password)
                .id(text_input::Id::new(ID_PASS))
                .on_input(Message::InputPass)
                .secure(true)
                .padding(10)
                .width(Length::Fill)
                .style(move |t, s| theme::input_style(colors, s))
                .on_submit(Message::ButtonConnection),
        ]
        .spacing(10),
    ]
    .spacing(15);

    // --- 4. BOUTONS D'ACTION ---
    let action_row = row![
        button(text("Démarrer SSH").width(Length::Fill).center())
            .on_press(Message::ButtonConnection)
            .padding(12)
            .width(Length::Fill)
            .style(move |t, s| theme::button_style(colors, s)),
        button(text("Enregistrer").width(Length::Fill).center())
            .on_press(Message::SaveSession)
            .padding(12)
            .width(Length::Fixed(120.0))
            .style(move |t, s| theme::button_style(colors, s)),
        button(text("🗑").width(Length::Fill).center())
            .on_press(Message::DeleteSession)
            .padding(12)
            .width(Length::Fixed(50.0))
            .style(move |t, s| theme::button_style(colors, s)),
    ]
    .spacing(10);

    // --- CONSTRUCTION FINALE ---
    container(
        column![
            text("Rust-PuTTY Manager")
                .size(30)
                .color(colors.accent)
                .width(Length::Fill)
                .center(),
            // Recherche
            text_input("🔍 Rechercher par nom, groupe, IP...", &app.search_query)
                .on_input(Message::SearchChanged)
                .padding(12)
                .style(move |t, s| theme::input_style(colors, s)),
            // Table
            column![
                table_header,
                scrollable(table_content).height(Length::Fixed(220.0)),
            ],
            horizontal_rule(2),
            // Formulaire
            encoding_form,
            action_row,
            // Logs
            scrollable(text(&app.logs).size(12).color(colors.text)).height(Length::Fixed(60.0))
        ]
        .spacing(20)
        .padding(20)
        .max_width(850),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(move |_| theme::main_container_style(colors))
    .into()
}

pub fn update(app: &mut MyApp, message: Message) -> Task<Message> {
    match message {
        Message::InputIP(ip) => app.current_session.ip = ip,
        Message::InputPort(port) => app.current_session.port = port,
        Message::InputUsername(u) => app.current_session.username = u,
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
        Message::SessionSelected(session) => {
            app.current_session.ip = session.ip.clone();
            app.current_session.username = session.username.clone();
            app.selected_session = Some(session);
        }
        Message::InputNewSessionName(name) => {
            app.current_session.name = name;
        }
        Message::InputNewSessionGroup(group) => {
            app.current_session.group = group;
        }
        Message::SearchChanged(query) => {
            app.search_query = query;
        }
        Message::SaveSession => {
            if !app.current_session.ip.is_empty() && !app.current_session.name.is_empty() {
                let group = if app.current_session.group.is_empty() {
                    "DEFAUT".to_string()
                } else {
                    app.current_session.group.to_uppercase()
                };

                let new_session = Session {
                    name: app.current_session.name.clone(),
                    ip: app.current_session.ip.clone(),
                    port: app.current_session.port.clone(),
                    username: app.current_session.username.clone(),
                    group,
                };
                app.sessions.push(new_session);
                // On trie par groupe puis par nom pour avoir une liste propre
                app.sessions
                    .sort_by(|a, b| a.group.cmp(&b.group).then(a.name.cmp(&b.name)));
            }
        }
        Message::DeleteSession => {
            if let Some(selected) = &app.selected_session {
                app.sessions.retain(|s| s != selected);
                app.selected_session = None;
                // Optionnel : vider les champs après suppression
                app.current_session.ip.clear();
                app.current_session.username.clear();
            }
        }
        _ => {}
    }
    Task::none()
}
