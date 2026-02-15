use iced::widget::{button, container, text_input};
use iced::{Border, Color, Shadow};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThemeChoice {
    #[default]
    Slate,
    Matrix,
    Cyberpunk,
    Nord,       // Nouveau : Élégant et froid
    Solarized,  // Nouveau : Classique des développeurs
    Dracula,    // Nouveau : Le célèbre thème sombre
    Tos,        // Nouveau : Style vieux BIOS / Amstrad
    Gruvbox,    // Tons terreux, ambiance "Old School"
    TokyoNight, // Bleu profond et néons roses/violets
    Coffee,     // Tons marrons et crème, très doux
    Ghost,      // Minimaliste, nuances de gris et blanc pur
    Catppuccin, // Le nouveau standard (très doux/pastel)
    Everforest, // Verts organiques, ultra-reposant
    RoséPine,   // Tons sourds, très "design"
    AyuMirage,  // Un entre-deux parfait, moderne et lisible
}

impl ThemeChoice {
    pub const ALL: [ThemeChoice; 15] = [
        ThemeChoice::Slate,
        ThemeChoice::Matrix,
        ThemeChoice::Cyberpunk,
        ThemeChoice::Nord,
        ThemeChoice::Dracula,
        ThemeChoice::Solarized,
        ThemeChoice::Tos,
        ThemeChoice::Gruvbox,
        ThemeChoice::TokyoNight,
        ThemeChoice::Coffee,
        ThemeChoice::Ghost,
        ThemeChoice::Catppuccin,
        ThemeChoice::Everforest,
        ThemeChoice::RoséPine,
        ThemeChoice::AyuMirage,
    ];

    pub fn get_colors(&self) -> TerminalColors {
        match self {
            ThemeChoice::Slate => TerminalColors {
                bg: Color::from_rgb(0.11, 0.13, 0.17),
                text: Color::from_rgb(0.85, 0.87, 0.91),
                prompt: Color::from_rgb(0.5, 0.88, 0.75),
                accent: Color::from_rgb(0.4, 0.7, 1.0),
                surface: Color::from_rgb(0.15, 0.17, 0.22),
            },
            ThemeChoice::Matrix => TerminalColors {
                bg: Color::from_rgb(0.02, 0.04, 0.02),
                text: Color::from_rgb(0.0, 0.8, 0.0),
                prompt: Color::from_rgb(0.0, 1.0, 0.0),
                accent: Color::from_rgb(0.0, 0.6, 0.0),
                surface: Color::from_rgb(0.05, 0.1, 0.05),
            },
            ThemeChoice::Cyberpunk => TerminalColors {
                bg: Color::from_rgb(0.05, 0.02, 0.08),
                text: Color::from_rgb(0.0, 1.0, 1.0),
                prompt: Color::from_rgb(1.0, 0.0, 1.0),
                accent: Color::from_rgb(1.0, 0.8, 0.0),
                surface: Color::from_rgb(0.1, 0.05, 0.15),
            },
            ThemeChoice::Nord => TerminalColors {
                bg: Color::from_rgb(0.18, 0.2, 0.25),      // Polar Night
                text: Color::from_rgb(0.88, 0.91, 0.94),   // Snow Storm
                prompt: Color::from_rgb(0.53, 0.75, 0.82), // Frost Blue
                accent: Color::from_rgb(0.51, 0.63, 0.76), // Arctic Blue
                surface: Color::from_rgb(0.23, 0.26, 0.32),
            },
            ThemeChoice::Dracula => TerminalColors {
                bg: Color::from_rgb(0.16, 0.17, 0.23),     // Dark
                text: Color::from_rgb(0.95, 0.95, 0.96),   // Selection
                prompt: Color::from_rgb(0.31, 0.98, 0.48), // Green
                accent: Color::from_rgb(0.74, 0.57, 0.97), // Purple
                surface: Color::from_rgb(0.26, 0.27, 0.35),
            },
            ThemeChoice::Solarized => TerminalColors {
                bg: Color::from_rgb(0.0, 0.17, 0.21),      // Base03
                text: Color::from_rgb(0.51, 0.58, 0.58),   // Base0
                prompt: Color::from_rgb(0.71, 0.54, 0.0),  // Yellow
                accent: Color::from_rgb(0.15, 0.45, 0.74), // Blue
                surface: Color::from_rgb(0.03, 0.21, 0.26),
            },
            ThemeChoice::Tos => TerminalColors {
                bg: Color::from_rgb(0.0, 0.0, 0.75),    // Bleu pur "vieux BIOS"
                text: Color::from_rgb(1.0, 1.0, 1.0),   // Blanc
                prompt: Color::from_rgb(1.0, 1.0, 0.0), // Jaune vif
                accent: Color::from_rgb(0.0, 1.0, 1.0), // Cyan
                surface: Color::from_rgb(0.0, 0.0, 0.5),
            },
            ThemeChoice::Gruvbox => TerminalColors {
                bg: Color::from_rgb(0.15, 0.15, 0.15),     // Dark 0
                text: Color::from_rgb(0.92, 0.86, 0.7),    // Light 1
                prompt: Color::from_rgb(0.72, 0.73, 0.14), // Green
                accent: Color::from_rgb(0.83, 0.36, 0.11), // Orange
                surface: Color::from_rgb(0.2, 0.2, 0.2),
            },
            ThemeChoice::TokyoNight => TerminalColors {
                bg: Color::from_rgb(0.06, 0.06, 0.09),     // Night
                text: Color::from_rgb(0.65, 0.7, 0.85),    // Storm
                prompt: Color::from_rgb(0.73, 0.58, 0.95), // Purple
                accent: Color::from_rgb(1.0, 0.46, 0.65),  // Pink
                surface: Color::from_rgb(0.1, 0.1, 0.15),
            },
            ThemeChoice::Coffee => TerminalColors {
                bg: Color::from_rgb(0.23, 0.18, 0.15),     // Mocha
                text: Color::from_rgb(0.93, 0.89, 0.85),   // Latte
                prompt: Color::from_rgb(0.76, 0.6, 0.42),  // Caramel
                accent: Color::from_rgb(0.55, 0.45, 0.35), // Espresso
                surface: Color::from_rgb(0.3, 0.25, 0.2),
            },
            ThemeChoice::Ghost => TerminalColors {
                bg: Color::from_rgb(0.02, 0.02, 0.02),   // Black
                text: Color::from_rgb(0.95, 0.95, 0.95), // White
                prompt: Color::from_rgb(0.4, 0.4, 0.4),  // Gray
                accent: Color::from_rgb(0.8, 0.8, 0.8),  // Light Gray
                surface: Color::from_rgb(0.1, 0.1, 0.1),
            },
            ThemeChoice::Catppuccin => TerminalColors {
                bg: Color::from_rgb(0.11, 0.11, 0.17),     // Mocha Base
                text: Color::from_rgb(0.8, 0.84, 0.95),    // Text (bleuté très doux)
                prompt: Color::from_rgb(0.8, 0.95, 0.75),  // Green (pastel)
                accent: Color::from_rgb(0.79, 0.72, 0.96), // Lavender
                surface: Color::from_rgb(0.12, 0.12, 0.19),
            },
            ThemeChoice::Everforest => TerminalColors {
                bg: Color::from_rgb(0.17, 0.2, 0.18), // Vert forêt sombre et mat
                text: Color::from_rgb(0.83, 0.82, 0.72), // Crème (zéro fatigue)
                prompt: Color::from_rgb(0.64, 0.75, 0.5), // Vert sauge
                accent: Color::from_rgb(0.89, 0.7, 0.44), // Orange sourd
                surface: Color::from_rgb(0.2, 0.23, 0.21),
            },
            ThemeChoice::RoséPine => TerminalColors {
                bg: Color::from_rgb(0.07, 0.07, 0.1), // Base (bleu-nuit profond)
                text: Color::from_rgb(0.88, 0.88, 0.95), // Rose pétale très clair
                prompt: Color::from_rgb(0.96, 0.74, 0.74), // Rose/Rouge sourd
                accent: Color::from_rgb(0.61, 0.8, 0.85), // Mousse
                surface: Color::from_rgb(0.1, 0.1, 0.15),
            },
            ThemeChoice::AyuMirage => TerminalColors {
                bg: Color::from_rgb(0.1, 0.13, 0.18),      // Gris-bleu équilibré
                text: Color::from_rgb(0.8, 0.8, 0.8),      // Gris neutre
                prompt: Color::from_rgb(1.0, 0.8, 0.44),   // Orange Ayu
                accent: Color::from_rgb(0.36, 0.74, 0.85), // Bleu Ayu
                surface: Color::from_rgb(0.14, 0.17, 0.23),
            },
        }
    }
}

impl fmt::Display for ThemeChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThemeChoice::Slate => write!(f, "Ardoise"),
            ThemeChoice::Matrix => write!(f, "Matrix"),
            ThemeChoice::Cyberpunk => write!(f, "Cyberpunk"),
            ThemeChoice::Nord => write!(f, "Nordique"),
            ThemeChoice::Solarized => write!(f, "Solarized"),
            ThemeChoice::Dracula => write!(f, "Dracula"),
            ThemeChoice::Tos => write!(f, "Amstrad/BIOS"),
            ThemeChoice::Gruvbox => write!(f, "Gruvbox"),
            ThemeChoice::TokyoNight => write!(f, "Tokyo Night"),
            ThemeChoice::Coffee => write!(f, "Café"),
            ThemeChoice::Ghost => write!(f, "Fantôme (Minimal)"),
            ThemeChoice::Catppuccin => write!(f, "Catppuccin (Doux)"),
            ThemeChoice::Everforest => write!(f, "Everforest (Reposant)"),
            ThemeChoice::RoséPine => write!(f, "Rosé Pine"),
            ThemeChoice::AyuMirage => write!(f, "Ayu Mirage"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TerminalColors {
    pub bg: Color,
    pub text: Color,
    pub prompt: Color,
    pub accent: Color,
    pub surface: Color,
}


// Default == Slate Theme
impl Default for TerminalColors {
    fn default() -> Self {
        Self {
            bg: Color::from_rgb(0.11, 0.13, 0.17),
            text: Color::from_rgb(0.85, 0.87, 0.91),
            prompt: Color::from_rgb(0.5, 0.88, 0.75),
            accent: Color::from_rgb(0.4, 0.7, 1.0),
            surface: Color::from_rgb(0.15, 0.17, 0.22),
        }
    }
}

#[derive(Clone, Copy)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Danger, // Optionnel : pour le bouton "Supprimer"
}


// --- FONCTIONS DE STYLE ---

pub fn button_style(
    colors: TerminalColors,
    status: button::Status,
    variant: ButtonVariant,
) -> button::Style {
    // 1. On définit les couleurs selon la variante
    let (bg_base, txt_color) = match variant {
        ButtonVariant::Primary => (colors.accent, colors.bg),
        ButtonVariant::Secondary => (colors.surface, colors.text),
        ButtonVariant::Danger => (Color::from_rgb(0.8, 0.2, 0.2), Color::WHITE),
    };

    // 2. On ajuste l'intensité selon l'état (Survol / Clic)
    let final_bg = match status {
        button::Status::Hovered => Color { a: 1.0, ..bg_base },
        button::Status::Pressed => Color { a: 0.7, ..bg_base },
        _ => Color { a: 0.85, ..bg_base }, // Un peu plus doux au repos
    };

    button::Style {
        background: Some(final_bg.into()),
        text_color: txt_color,
        border: Border {
            radius: 6.0.into(),
            width: if let ButtonVariant::Secondary = variant { 1.0 } else { 0.0 },
            color: colors.accent,
        },
        ..Default::default()
    }
}

pub fn input_style(colors: TerminalColors, status: text_input::Status) -> text_input::Style {
    let border_color = match status {
        text_input::Status::Focused => colors.accent,
        _ => Color::from_rgba(1.0, 1.0, 1.0, 0.1),
    };

    text_input::Style {
        background: colors.surface.into(),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 4.0.into(),
        },
        value: colors.text,
        placeholder: Color::from_rgb(0.4, 0.4, 0.4),
        selection: colors.accent,
        icon: Color::TRANSPARENT,
    }
}

pub fn main_container_style(colors: TerminalColors) -> container::Style {
    container::Style {
        background: Some(colors.bg.into()),
        text_color: Some(colors.text),
        ..Default::default()
    }
}

