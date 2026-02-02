use std::os::linux::raw;

use crate::ui::theme::{self, ThemeChoice};
use crate::ui::{MAX_TERMINAL_LINES, Message, MyApp, SCROLLABLE_ID};

use iced::widget::{button, column, container, pick_list, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length, Task};
use vt100;

pub fn view(app: &MyApp) -> Element<'_, Message> {
    let colors = app.current_profile.theme.get_colors();

    // --- 1. BARRE D'ONGLETS & SÉLECTEUR ---
    let tab_bar = container(
        row![
            container(
                text(format!(" 🐚 {} ", &app.current_profile.ip))
                    .size(13)
                    .font(iced::Font::MONOSPACE)
                    .color(colors.text)
            )
            .padding([6, 18])
            .style(move |_| container::Style {
                background: Some(colors.bg.into()),
                border: iced::Border {
                    width: 1.0,
                    color: colors.accent,
                    radius: iced::border::Radius {
                        top_left: 6.0,
                        top_right: 6.0,
                        ..Default::default()
                    },
                },
                ..Default::default()
            }),
            pick_list(
                &ThemeChoice::ALL[..],
                Some(app.current_profile.theme),
                Message::ThemeSelected
            )
            .text_size(12)
            .padding(5),
            button(text("+").size(16)).style(button::text).padding(10),
        ]
        .spacing(15)
        .align_y(Alignment::Center), //.padding([8, 12, 0, 12]),
    )
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(colors.surface.into()),
        ..Default::default()
    });

    // --- 2. ZONE DE LOGS (DIAGNOSTIC TEST) ---
    let screen = app.parser.screen();
    let (rows, cols) = screen.size();
    let (cursor_row, cursor_col) = screen.cursor_position();

    let terminal_logs = scrollable(
        container(
            column(
                (0..rows)
                    .map(|row_idx| {
                        let mut line_elements = Vec::new();
                        let mut current_text = String::new();
                        let mut current_fg = vt100::Color::Default;

                        // --- LE TEST ULTIME : On ajoute un numéro de ligne à gauche ---
                        // Si tu vois "ss" à côté du même numéro de ligne, le serveur fait un echo.
                        line_elements.push(
                            text(format!("{:2} | ", row_idx))
                                .size(12)
                                .color(colors.accent.clone()) // Utilise une couleur visible
                                .font(iced::Font::MONOSPACE)
                                .into(),
                        );

                        for col_idx in 0..cols {
                            let is_cursor =
                                row_idx as u16 == cursor_row && col_idx as u16 == cursor_col;

                            if let Some(cell) = screen.cell(row_idx, col_idx) {
                                let fg = cell.fgcolor();
                                let content = cell.contents();

                                // On traite le contenu : si vide et pas curseur, un espace suffit
                                let display_char = if content.is_empty() { " " } else { &content };

                                // Si le style change ou qu'on arrive au curseur, on vide le buffer
                                if (fg != current_fg || is_cursor) && !current_text.is_empty() {
                                    line_elements.push(
                                        text(current_text.clone())
                                            .font(iced::Font::MONOSPACE)
                                            .size(15)
                                            .color(vt_to_iced_color(current_fg, &colors))
                                            .into(),
                                    );
                                    current_text.clear();
                                }

                                if is_cursor {
                                    line_elements.push(
                                        container(
                                            text(display_char.to_string())
                                                .font(iced::Font::MONOSPACE)
                                                .size(15)
                                                .color(colors.bg),
                                        )
                                        .style(move |_| container::Style {
                                            background: Some(vt_to_iced_color(fg, &colors).into()),
                                            ..Default::default()
                                        })
                                        .into(),
                                    );
                                    // Après le curseur, on repart sur la couleur de la cellule
                                    current_fg = fg;
                                } else {
                                    current_fg = fg;
                                    current_text.push_str(display_char);
                                }
                            }
                        }

                        // On vide le dernier morceau de texte de la ligne
                        if !current_text.is_empty() {
                            line_elements.push(
                                text(current_text)
                                    .font(iced::Font::MONOSPACE)
                                    .size(15)
                                    .color(vt_to_iced_color(current_fg, &colors))
                                    .into(),
                            );
                        }

                        row(line_elements).spacing(0).into()
                    })
                    .collect::<Vec<_>>(),
            )
            .spacing(0),
        )
        .padding(25.0)
        .width(Length::Fill),
    )
    .id(scrollable::Id::new(SCROLLABLE_ID))
    .height(Length::Fill);
    // --- 3. BARRE D'ÉTAT ---
    let status_bar = container(
        row![
            container(
                text(format!(" ● CONNECTED: {} ", app.current_profile.username))
                    .size(11)
                    .color(colors.bg)
            )
            .padding([3, 10])
            .style(move |_| container::Style {
                background: Some(colors.prompt.into()),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
            text(format!(" {}x{} ", cols, rows))
                .size(11)
                .color(colors.accent)
                .font(iced::Font::MONOSPACE)
        ]
        .spacing(12)
        .align_y(Alignment::Center)
        .padding(10),
    )
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(colors.surface.into()),
        ..Default::default()
    });

    column![tab_bar, terminal_logs, status_bar].into()
}

pub fn update(app: &mut MyApp, message: Message) -> Task<Message> {
    match message {
        Message::ThemeSelected(new_theme) => {
            app.current_profile.theme = new_theme;
        }

        // raw_bytes est maintenant un Vec<u8> (les données brutes du SSH)
        Message::SshData(raw_bytes) => {
            println!("Data reçue : {:?}", String::from_utf8_lossy(&raw_bytes));
            app.parser.process(&raw_bytes);

            let (r, c) = app.parser.screen().size();
            println!("Taille du parser avant process : {} rows, {} cols", r, c);

            let pos = app.parser.screen().cursor_position();
            println!("Position curseur après : {:?}", pos);

            // On envoie la commande de scroll tout en bas
            return scrollable::snap_to::<Message>(
                scrollable::Id::new(SCROLLABLE_ID),
                scrollable::RelativeOffset::END,
            );
        }

        Message::SetChannel(ch) => {
            app.active_channel = Some(ch);
            //app.logs.push_str(">> Session SSH établie\n");
        }

        Message::KeyPressed(key, modifiers) => {
            if let Some(channel) = &app.active_channel {
                let mut to_send = None;

                println!("DEBUG Clavier: Key={:?}, Modifiers={:?}", key, modifiers);
                match key {
                    // 1. GESTION UNIVERSELLE DES CARACTÈRES (Lettres, Chiffres, Point, Slash, etc.)
                    iced::keyboard::Key::Character(c) => {
                        if modifiers.control() {
                            let char_bytes = c.as_bytes();
                            if !char_bytes.is_empty() {
                                to_send = Some(vec![char_bytes[0] & 0x1f]);
                            }
                        } else {
                            // Ici, on capture TOUT : "a", "1", ".", "/", etc.

                            // --- CORRECTIF AZERTY ---
                            // Si on a Maj + ; et qu'Iced renvoie ";", on force le "."
                            let char_to_send = if modifiers.shift() && c == ";" {
                                ".".to_string()
                            } else if modifiers.shift() && c == ":" {
                                "/".to_string() // Souvent le même problème pour le slash
                            } else {
                                c.to_string()
                            };
                            to_send = Some(char_to_send.as_bytes().to_vec());
                        }
                    }

                    // 2. TOUCHES NOMMÉES (Touches qui n'ont pas de représentation textuelle directe)
                    iced::keyboard::Key::Named(named) => match named {
                        iced::keyboard::key::Named::Space => to_send = Some(b" ".to_vec()),
                        iced::keyboard::key::Named::Enter => to_send = Some(b"\r".to_vec()),
                        iced::keyboard::key::Named::Backspace => to_send = Some(b"\x7f".to_vec()),
                        iced::keyboard::key::Named::Tab => to_send = Some(b"\t".to_vec()),
                        iced::keyboard::key::Named::Escape => to_send = Some(b"\x1b".to_vec()),
                        iced::keyboard::key::Named::ArrowUp => to_send = Some(b"\x1b[A".to_vec()),
                        iced::keyboard::key::Named::ArrowDown => to_send = Some(b"\x1b[B".to_vec()),
                        iced::keyboard::key::Named::ArrowRight => {
                            to_send = Some(b"\x1b[C".to_vec())
                        }
                        iced::keyboard::key::Named::ArrowLeft => to_send = Some(b"\x1b[D".to_vec()),
                        _ => {}
                    },
                    _ => {}
                }

                if let Some(bytes) = to_send {
                    let ch_clone = channel.clone();
                    return Task::perform(
                        async move {
                            if let Ok(mut h) = ch_clone.try_lock() {
                                let _ = h.data(bytes.as_slice()).await;
                            }
                        },
                        |_| Message::DoNothing,
                    );
                }
            }
            //Task::none()
        }

        Message::RawKey(bytes) => {
            if let Some(channel) = &app.active_channel {
                let ch_clone = channel.clone();
                return Task::perform(
                    async move {
                        if let Ok(mut h) = ch_clone.try_lock() {
                            let _ = h.data(bytes.as_slice()).await;
                        }
                    },
                    |_| Message::DoNothing,
                );
            }
            //Task::none()
        }

        _ => {}
    }
    Task::none()
}

fn vt_to_iced_color(
    vt_color: vt100::Color,
    theme_colors: &crate::ui::theme::TerminalColors,
) -> iced::Color {
    match vt_color {
        vt100::Color::Default => theme_colors.text, // Utilise la couleur de ton thème
        //vt100::Color::Default => iced::Color::from_rgb8(255, 255, 255),
        vt100::Color::Idx(i) => {
            // Conversion basique des 16 premières couleurs ANSI
            match i {
                0 => iced::Color::from_rgb8(0, 0, 0),       // Black
                1 => iced::Color::from_rgb8(205, 0, 0),     // Red
                2 => iced::Color::from_rgb8(0, 205, 0),     // Green
                3 => iced::Color::from_rgb8(205, 205, 0),   // Yellow
                4 => iced::Color::from_rgb8(0, 0, 238),     // Blue
                5 => iced::Color::from_rgb8(205, 0, 205),   // Magenta
                6 => iced::Color::from_rgb8(0, 205, 205),   // Cyan
                7 => iced::Color::from_rgb8(229, 229, 229), // White
                _ => theme_colors.text,
            }
        }
        vt100::Color::Rgb(r, g, b) => iced::Color::from_rgb8(r, g, b),
    }
}
