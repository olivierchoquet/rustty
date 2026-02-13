use std::os::linux::raw;

use crate::messages::{ConfigMessage, SshMessage};
use crate::ui::theme::{self, TerminalColors, ThemeChoice};
use crate::ui::{
    ID_IP, ID_PASS, ID_PORT, ID_USER, MAX_TERMINAL_LINES, Message, MyApp, SCROLLABLE_ID,
};

use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};
use iced::widget::{button, column, container, pick_list, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length, Task};
use vt100;

pub fn render(app: &MyApp, window_id: iced::window::Id) -> Element<'_, Message> {
    let colors = app.current_profile.theme.get_colors();

// --- CHANGEMENT ICI ---
    // On va chercher le parser spécifique à cette fenêtre.pub fn render(app: &MyApp, window_id: iced::window::Id) -> Element<'_, Message> {
    // Si on ne le trouve pas, on affiche un message d'attente.
    let Some(parser) = app.parsers.get(&window_id) else {
        return container(text("Connexion en cours...").color(colors.text))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(move |_| container::Style {
                background: Some(colors.bg.into()),
                ..Default::default()
            })
            .into();
    };

    let screen = parser.screen();
    let (rows, cols) = screen.size();
    let (cursor_row, cursor_col) = screen.cursor_position();

    // On pré-clône les couleurs pour les closures de la barre d'onglets et d'état
    let tab_colors = colors.clone();
    let status_colors = colors.clone();
    let bg_color_final = colors.bg;

    // --- 1. BARRE D'ONGLETS (Header) ---
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
                background: Some(tab_colors.bg.into()),
                border: iced::Border {
                    width: 1.0,
                    color: tab_colors.accent,
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
                |theme| Message::Config(ConfigMessage::ThemeChanged(theme)) // Correction ici
            )
            .text_size(12)
            .padding(5),
            button(text("+").size(16))
                .style(iced::widget::button::text)
                .padding(10),
        ]
        .spacing(15)
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(tab_colors.surface.into()),
        ..Default::default()
    });

    // --- 2. ZONE TERMINAL ---
    let terminal_content = column(
        (0..rows)
            .map(|row_idx| {
                let mut line_elements = Vec::new();
                let mut current_text = String::new();
                let mut current_fg = vt100::Color::Default;

                for col_idx in 0..cols {
                    let is_cursor = row_idx as u16 == cursor_row && col_idx as u16 == cursor_col;

                    if let Some(cell) = screen.cell(row_idx, col_idx) {
                        let fg = cell.fgcolor();
                        let content = cell.contents();
                        let display_char = if content.is_empty() {
                            " "
                        } else {
                            content.as_str()
                        };

                        if (fg != current_fg || is_cursor) && !current_text.is_empty() {
                            // On passe colors par valeur (cloné)
                            line_elements.push(render_text_chunk(
                                current_text.clone(),
                                current_fg,
                                colors.clone(),
                            ));
                            current_text.clear();
                        }

                        if is_cursor {
                            line_elements.push(render_cursor(
                                display_char.to_string(),
                                fg,
                                colors.clone(),
                            ));
                            current_fg = fg;
                        } else {
                            current_fg = fg;
                            current_text.push_str(display_char);
                        }
                    }
                }

                if !current_text.is_empty() {
                    line_elements.push(render_text_chunk(current_text, current_fg, colors.clone()));
                }

                row(line_elements).spacing(0).into()
            })
            .collect::<Vec<_>>(),
    )
    .spacing(0);

    let terminal_scroll = scrollable(
        container(terminal_content)
            .padding(20)
            .width(Length::Fill)
            .style(move |_| container::Style {
                background: Some(bg_color_final.into()),
                ..Default::default()
            }),
    )
    //.id(scrollable::Id::new(SCROLLABLE_ID))
    .id(scrollable::Id::new(format!("scroll_{:?}", window_id)))
    .height(Length::Fill);

    // --- AJOUT DU FOCUS AU CLIC ---
    // On enveloppe le scrollable dans un bouton transparent qui couvre toute la zone
    let interactive_terminal = button(terminal_scroll)
    .padding(0)
    .style(iced::widget::button::secondary) // Ou un style transparent personnalisé
    .on_press(Message::Ssh(SshMessage::WindowFocused(window_id)));

    // --- 3. BARRE D'ÉTAT (Footer) ---
    let status_bar = container(
        row![
            container(
                text(format!(" ● {} ", app.current_profile.username))
                    .size(11)
                    .color(status_colors.bg)
            )
            .padding([3, 10])
            .style(move |_| container::Style {
                background: Some(status_colors.prompt.into()),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
            text(format!(" Terminal: {}x{} ", cols, rows))
                .size(11)
                .color(status_colors.accent)
                .font(iced::Font::MONOSPACE)
        ]
        .spacing(12)
        .align_y(Alignment::Center)
        .padding(10),
    )
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(status_colors.surface.into()),
        ..Default::default()
    });

    column![tab_bar, interactive_terminal, status_bar].into()
}

// --- HELPERS

fn render_text_chunk(
    txt: String,
    vt_color: vt100::Color,
    colors: TerminalColors,
) -> Element<'static, Message> {
    text(txt)
        .font(iced::Font::MONOSPACE)
        .size(15)
        .color(vt_to_iced_color(vt_color, &colors))
        .into()
}

fn render_cursor(
    char_str: String,
    vt_color: vt100::Color,
    colors: TerminalColors,
) -> Element<'static, Message> {
    let cursor_bg = vt_to_iced_color(vt_color, &colors);
    let cursor_fg = colors.bg;

    container(
        text(char_str)
            .font(iced::Font::MONOSPACE)
            .size(15)
            .color(cursor_fg),
    )
    .style(move |_| container::Style {
        background: Some(cursor_bg.into()),
        ..Default::default()
    })
    .into()
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

