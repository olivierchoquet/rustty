use crate::messages::{Message, ProfileMessage};
use crate::ui::theme;
use crate::ui::{MyApp, theme::TerminalColors};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Font, Length, font};

pub fn header<'a>(app: &'a MyApp, colors: TerminalColors) -> Element<'a, Message> {
    column![

        text_input("🔍 Recherche rapide sur nom, groupe, ip, utilisateur", &app.search_query)
            .on_input(|v| Message::Profile(ProfileMessage::SearchChanged(v)))
            .padding(10)
            .style(move |theme: &iced::Theme, status| {
                theme::input_style(colors, status)
            }),

        container(
            row![
                bold_text("GROUPE").width(Length::FillPortion(1)),
                bold_text("NOM").width(Length::FillPortion(2)),
                bold_text("UTILISATEUR").width(Length::FillPortion(1)),
                bold_text("ADRESSE IP").width(Length::FillPortion(2)),
            ]
            .spacing(10)
        )
        .padding(10)
        .style(move |_theme| {
            container::Style {
                background: Some(colors.bg.into()),
                text_color: Some(colors.text.into()),
                ..Default::default()
            }
        })
    ]
    .spacing(20) 
    .into()
}

// <'a> means that provided MyApp reference 
// must live at least as long as the produced UI element.
pub fn content<'a>(app: &'a MyApp, colors: TerminalColors) -> Element<'a, Message> {
    let mut content = column![].spacing(1);
    let query = app.search_query.to_lowercase();

    for (i, profile) in app.profiles.iter().enumerate() {
        let is_match = query.is_empty() 
            || profile.name.to_lowercase().contains(&query) 
            || profile.group.to_lowercase().contains(&query) 
            || profile.ip.contains(&query)          
            || profile.username.to_lowercase().contains(&query); 
        if is_match {
            let is_selected = app.selected_profile_id == Some(profile.id);
            let zebra_color = if i % 2 == 0 {
                colors.surface
            } else {
                colors.bg
            };
            content = content.push(
                button(
                    container(
                        row![
                            text(&profile.group).width(Length::FillPortion(1)),
                            text(&profile.name).width(Length::FillPortion(2)),
                            text(&profile.username).width(Length::FillPortion(1)),
                            text(format!("{}:{}", profile.ip, profile.port))
                                .width(Length::FillPortion(2)),
                        ]
                        .spacing(10),
                    )
                    .padding(8),
                )
                .width(Length::Fill)
                .on_press(Message::Profile(ProfileMessage::Selected(profile.id)))
                .style(move |_, status| {
                    let mut st = theme::button_style(colors, status,theme::ButtonVariant::Secondary);
                    if is_selected {
                        st.background = Some(colors.prompt.into());
                        st.text_color = colors.accent;
                        st.border.width = 2.0;
                        st.border.color = colors.accent;
                    } else {
                        st.background = Some(zebra_color.into());
                        st.text_color = colors.text;
                        st.border.width = 0.0;
                    }
                    st
                }),
            );
        }
    }
    scrollable(content).height(Length::Fixed(150.0)).into()
}


// helper for bold text in the header
fn bold_text(content: &str) -> text::Text {
    text(content).font(Font {
        weight: font::Weight::Bold,
        ..Default::default()
    })
}