use iced::{
    Alignment, Color, Element, Font, Length, Task,
    font::Weight,
    widget::{column, container, horizontal_rule, row, text, text_input, vertical_space},
};

use crate::{messages::{ConfigMessage, LoginMessage, Message, ProfileMessage}, ui::{EditSection, MyApp, Profile, components::{forms::{general_form, theme_form}, table::{content, header}}, theme}};
use crate::ui::components::{actions, sidebar};

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
    let actions_bar = actions::buttons_form(colors, app.selected_profile_id.is_some());
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

pub fn update(app: &mut MyApp, message: Message) -> Task<Message> {
    match message {
        Message::Login(LoginMessage::InputIP(ip)) => app.current_profile.ip = ip,
        Message::Login(LoginMessage::InputPort(port)) => app.current_profile.port = port,
        Message::Login(LoginMessage::InputUsername(u)) => app.current_profile.username = u,
        Message::Login(LoginMessage::InputPass(p)) => app.password = p,
    
        Message::Profile(ProfileMessage::Selected(id)) => {
            // 1. On cherche le profil dans notre liste grâce à l'ID
            if let Some(profile_trouve) = app.profiles.iter().find(|p| p.id == id) {
                // 2. On marque cet ID comme étant celui sélectionné (pour le surlignage du tableau)
                app.selected_profile_id = Some(id);

                // 3. On remplit le formulaire avec une copie des données
                // .clone() est nécessaire ici car current_profile possède ses propres données
                app.current_profile = profile_trouve.clone();

                println!(
                    "DEBUG: Profil '{}' chargé dans le formulaire",
                    profile_trouve.name
                );
            }
        }
        Message::Profile(ProfileMessage::InputName(name)) => {
            app.current_profile.name = name;
        }
        Message::Profile(ProfileMessage::InputGroup(group)) => {
            app.current_profile.group = group;
        }
        Message::Profile(ProfileMessage::SearchChanged(query)) => {
            app.search_query = query;
        }
        Message::Profile(ProfileMessage::Save) => {
            // 1. On vérifie les champs obligatoires
            if !app.current_profile.ip.is_empty() && !app.current_profile.name.is_empty() {
                // On gère le groupe par défaut
                if app.current_profile.group.is_empty() {
                    app.current_profile.group = "DEFAUT".to_string();
                } else {
                    app.current_profile.group = app.current_profile.group.to_uppercase();
                }

                match app.selected_profile_id {
                    // --- CAS UPDATE : Un profil est sélectionné ---
                    Some(id_recherche) => {
                        if let Some(index) = app.profiles.iter().position(|p| p.id == id_recherche)
                        {
                            // On remplace l'ancien profil par les nouvelles données du formulaire
                            // Tout en s'assurant que l'ID reste celui d'origine
                            let mut updated_profile = app.current_profile.clone();
                            updated_profile.id = id_recherche;

                            app.profiles[index] = updated_profile;
                            println!("DEBUG: Profil mis à jour à l'index {}", index);
                        }
                    }
                    // --- CAS INSERT : Aucun profil sélectionné ---
                    None => {
                        let mut new_profile = app.current_profile.clone();
                        new_profile.id = uuid::Uuid::new_v4(); // On génère un ID unique pour le nouveau
                        let new_profile_id = new_profile.id;

                        app.profiles.push(new_profile);
                        app.selected_profile_id = Some(new_profile_id); // On sélectionne le nouveau venu !
                        println!("DEBUG: Nouveau profil inséré dans la liste");
                    }
                }

                // 2. Post-traitement : Tri, Sauvegarde disque et Reset
                app.profiles
                    .sort_by(|a, b| a.group.cmp(&b.group).then(a.name.cmp(&b.name)));

                app.save_profiles();
            }
        }

        Message::Profile(ProfileMessage::New) => {
            app.selected_profile_id = None; // On sort du mode édition
            app.current_profile = Profile::default(); // On vide les champs (formulaire vierge)
            println!("DEBUG: Formulaire réinitialisé pour un nouveau profil");
        }

        Message::Profile(ProfileMessage::Delete) => {
            if let Some(selected_profile_id) = &app.selected_profile_id {
                app.profiles.retain(|p| p.id != *selected_profile_id);
                app.selected_profile_id = None;
                // Optionnel : vider les champs après suppression
                app.current_profile.ip.clear();
                app.current_profile.username.clear();
                app.save_profiles(); // On écrit sur le disque après la suppression
            }
        }
        Message::Config(ConfigMessage::SectionChanged(section)) => {
            println!("Changement de section vers : {:?}", section);
            app.active_section = section;
            // Iced 0.13 redessine automatiquement dès que l'état change
        }
        Message::Config(ConfigMessage::ThemeChanged(new_theme)) => {
            app.current_profile.theme = new_theme;
            // Optionnel : println!("Thème appliqué : {:?}", new_theme);

            // Si tu as une fonction de sauvegarde, c'est le moment
            // de sauver le choix du thème pour le prochain démarrage.
            app.save_profiles();
        }
        _ => {}
    }
    Task::none()
}
