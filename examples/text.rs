use iced_futures::executor::Tokio;
use iced_native::Subscription;
use iced_native::{
    keyboard, subscription, Color, Column, Command, Container, Element, Event, HorizontalAlignment,
    Length, Row, Text,
};
use iced_tui::{Application, Style, TuiRenderer};
use simplelog::{Config, LevelFilter, WriteLogger};

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
        self.should_exit
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        subscription::events().map(Self::Message::EventOccurred)
    }

    fn view(&self) -> Element<'_, Self::Message, TuiRenderer> {
        Container::new(
            Row::new()
                .spacing(4)
                .width(Length::Shrink)
                .push(
                    Column::new()
                        .spacing(4)
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
                        .width(Length::Shrink),
                )
                .push(
                    Column::new()
                        .spacing(4)
                        .push(Container::new(
                            Text::new("Hello 2 test iced-tui!\nThis is a toy renderer")
                                .font(Style::default().fg(Color {
                                    r: 0.3,
                                    g: 0.8,
                                    b: 0.,
                                    a: 1.,
                                }))
                                .width(Length::Shrink)
                                .horizontal_alignment(HorizontalAlignment::Center),
                        ))
                        .push(Text::new("Other text 2").width(Length::Shrink))
                        .width(Length::Shrink),
                ),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .style(Style::default().bg(Color {
            r: 0.2,
            g: 0.2,
            b: 0.2,
            a: 1.,
        }))
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
    WriteLogger::init(LevelFilter::Debug, Config::default(), std::io::stderr()).unwrap();
    MyApp::run()
}
