use iced_futures::executor::Tokio;
use iced_native::Subscription;
use iced_native::{
    keyboard, subscription, Color, Column, Command, Container, Element, Event, HorizontalAlignment,
    Length, Text,
};
use iced_tui::{Application, TuiRenderer};

pub struct MyApp {
    should_exit: Option<u8>,
}

#[derive(Clone, Debug)]
pub enum AppMessage {
    EventOccurred(Event),
}

impl Application for MyApp {
    type Message = AppMessage;
    type Executor = Tokio;

    fn new() -> (MyApp, Command<Self::Message>) {
        (MyApp { should_exit: None }, Command::none())
    }

    fn should_exit(&self) -> Option<u8> {
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
                    Text::new("Hello test iced-tui!\nThis is a toy renderer")
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
                .width(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            AppMessage::EventOccurred(Event::Keyboard(keyboard::Event::KeyReleased {
                key_code,
                modifiers,
            })) => {
                if key_code == keyboard::KeyCode::Q {
                    // exit on press q (status 0 = success)
                    self.should_exit = Some(0);
                }

                if key_code == keyboard::KeyCode::C && modifiers.control {
                    // exit on ctrl+c (status 1 = error)
                    self.should_exit = Some(1);
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
