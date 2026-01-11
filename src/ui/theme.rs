use iced::{Color, Border, Shadow};
use iced::widget::{text_input, button, container};

// --- TA PALETTE DE COULEURS ---
pub const COLOR_BG: Color = Color::from_rgb(0.11, 0.13, 0.17);    // Gris-bleu ardoise (plus doux)
pub const COLOR_TEXT: Color = Color::from_rgb(0.85, 0.87, 0.91);  // Blanc bleuté
pub const COLOR_PROMPT: Color = Color::from_rgb(0.5, 0.88, 0.75);  // Vert menthe pastel
pub const COLOR_ACCENT: Color = Color::from_rgb(0.4, 0.7, 1.0);   // Bleu ciel pour les bordures

// --- STYLE DES INPUTS (CHAMPS DE SAISIE) ---
pub fn input_style(_theme: &iced::Theme, status: text_input::Status) -> text_input::Style {
    let border_color = match status {
        text_input::Status::Focused => COLOR_ACCENT,           // Brille en bleu quand on clique dedans
        _ => Color::from_rgba(1.0, 1.0, 1.0, 0.1),            // Discret le reste du temps
    };

    text_input::Style {
        background: Color::from_rgb(0.1, 0.12, 0.15).into(),  // Un peu plus clair que le fond
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 4.0.into(),
        },
        value: COLOR_TEXT,
        placeholder: Color::from_rgb(0.4, 0.4, 0.4),
        selection: COLOR_ACCENT,
        icon: Color::TRANSPARENT, // Corrigé : plus de panic ici
    }
}

// --- STYLE DES BOUTONS ---
pub fn button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let base_color = if status == button::Status::Hovered {
        Color::from_rgb(0.0, 0.6, 1.0) // Plus clair au survol
    } else {
        COLOR_ACCENT
    };

    button::Style {
        background: Some(base_color.into()),
        text_color: Color::WHITE,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 4.0.into(),
        },
        shadow: Shadow::default(),
    }
}

// --- STYLE DU FOND DE L'APPLICATION ---
pub fn main_container_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(COLOR_BG.into()),
        text_color: Some(COLOR_TEXT),
        ..container::Style::default()
    }
}