/***** Contient uniquement la vue du formulaire de connexion  ******/
use iced::widget::{column, container, row, scrollable, text, text_input, button};
use iced::{Alignment, Element, Length, Border, Padding};
use crate::ui::{COLOR_ACCENT, COLOR_BG, COLOR_PROMPT, COLOR_TEXT, ID_IP, ID_PASS, ID_PORT, ID_USER, Message, MyApp, SCROLLABLE_ID};

pub fn view(app: &MyApp) -> Element<'_, Message> {
        container(
            column![
                text("Rust-PuTTY Login")
                    .size(32)
                    .color(iced::Color::from_rgb(0.2, 0.5, 0.9)),
                row![
                    text_input("IP", &app.ip)
                        .id(text_input::Id::new(ID_IP))
                        .on_input(Message::InputIP)
                        .padding(10)
                        .width(Length::FillPortion(3)),
                    text_input("Port", &app.port)
                        .id(text_input::Id::new(ID_PORT))
                        .on_input(Message::InputPort)
                        .padding(10)
                        .width(Length::FillPortion(1)),
                ]
                .spacing(10),
                text_input("Utilisateur", &app.username)
                    .id(text_input::Id::new(ID_USER))
                    .on_input(Message::InputUsername)
                    .padding(10),
                text_input("Mot de passe", &app.password)
                    .id(text_input::Id::new(ID_PASS))
                    .on_input(Message::InputPass)
                    .secure(true)
                    .padding(10)
                    .on_submit(Message::ButtonConnection), // Entrée ici lance la connexion
                button("Démarrer la session SSH")
                    .on_press(Message::ButtonConnection)
                    .padding(12)
                    .width(Length::Fill),
                scrollable(text(&app.logs).size(13)).height(Length::Fill)
            ]
            .spacing(15)
            .padding(30)
            .max_width(450),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }