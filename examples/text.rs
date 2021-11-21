use iced_futures::executor::Tokio;
use iced_native::Subscription;
use iced_native::{
    keyboard, subscription, Color, Column, Command, Container, Element, Event, HorizontalAlignment,
    Length, Text,
};
use iced_tui::{Application, TuiRenderer};

pub struct MyApp {
    username: String,
    should_exit: bool,
}

#[derive(Clone, Debug)]
pub enum AppMessage {
    EventOccurred(Event),
}

impl Application for MyApp {
    type Message = AppMessage;
    type Executor = Tokio;

    fn new() -> (MyApp, Command<Self::Message>) {
        (
            MyApp {
                username: "Test1".to_string(),
                should_exit: false,
            },
            Command::none(),
        )
    }

    fn should_exit(&self) -> bool {
        return self.should_exit;
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        subscription::events().map(Self::Message::EventOccurred)
    }

    fn view(&mut self) -> Element<'_, Self::Message, TuiRenderer> {
        Container::new(
            Column::new()
                .spacing(1)
                .push(
                    Text::new("Hello pancurses!\nThis is a toy renderer")
                        .color(Color {
                            r: 0.,
                            g: 0.,
                            b: 1.,
                            a: 1.,
                        })
                        .width(Length::Shrink)
                        .horizontal_alignment(HorizontalAlignment::Center),
                )
                .push(Text::new("Other text").width(Length::Shrink))
                .width(Length::Shrink),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        eprintln!(
            "{:?} - APP - update called for message: {:?}",
            std::thread::current().id(),
            message
        );

        match message {
            AppMessage::EventOccurred(Event::Keyboard(keyboard::Event::KeyPressed {
                key_code,
                modifiers,
            })) => {
                eprintln!(
                    "{:?} - APP - message matched: {:?}",
                    std::thread::current().id(),
                    message
                );

                if key_code == keyboard::KeyCode::C && modifiers.control {
                    self.should_exit = true;
                }

                Command::none()
            }
            _ => Command::none(),
        }
    }
}

fn main() {
    MyApp::run()
}
