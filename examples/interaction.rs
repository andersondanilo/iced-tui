use iced_futures::executor::Tokio;
use iced_native::Subscription;
use iced_native::{
    button, keyboard, subscription, text_input, Button, Color, Column, Command, Container, Element,
    Event, HorizontalAlignment, Length, Row, Space, Text, TextInput,
};
use iced_tui::{Application, Style, TuiRenderer};
use simplelog::{Config, LevelFilter, WriteLogger};

pub struct MyApp {
    should_exit: Option<u8>,
    text_state: text_input::State,
    input_value: String,
    button_state: button::State,
}

#[derive(Clone, Debug)]
pub enum AppMessage {
    EventOccurred(Event),
    InputValueChanged(String),
}

impl Application for MyApp {
    type Message = AppMessage;
    type Executor = Tokio;

    fn new() -> (MyApp, Command<Self::Message>) {
        let mut text_state = text_input::State::new();
        text_state.focus();
        (
            MyApp {
                should_exit: None,
                text_state,
                input_value: "".to_string(),
                button_state: button::State::default(),
            },
            Command::none(),
        )
    }

    fn should_exit(&self) -> Option<u8> {
        self.should_exit
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        subscription::events().map(Self::Message::EventOccurred)
    }

    fn view(&mut self) -> Element<'_, Self::Message, TuiRenderer> {
        Container::new(
            Column::new()
                .spacing(1)
                .width(Length::Shrink)
                .push(Text::new("Line 1"))
                .push(
                    Row::new()
                        .spacing(1)
                        .push(Text::new("Name: "))
                        .push(Space::new(Length::Units(3), Length::Units(1)))
                        .push(TextInput::new(
                            &mut self.text_state,
                            "Type something",
                            &self.input_value,
                            AppMessage::InputValueChanged,
                        ))
                        .push(Button::new(&mut self.button_state, Text::new(" Send "))),
                ),
        )
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
            AppMessage::EventOccurred(_) => Command::none(),
            AppMessage::InputValueChanged(value) => {
                self.input_value = value;
                Command::none()
            }
        }
    }
}

fn main() {
    WriteLogger::init(LevelFilter::Debug, Config::default(), std::io::stderr()).unwrap();
    MyApp::run()
}
