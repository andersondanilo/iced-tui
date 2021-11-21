use iced_futures::executor::Tokio;
use iced_native::{Color, Column, Command, Container, Element, HorizontalAlignment, Length, Text};
use iced_tui::{Application, TuiRenderer};

pub struct MyApp {
    username: String,
}

impl Application for MyApp {
    type Message = ();
    type Executor = Tokio;

    fn new() -> (MyApp, Command<Self::Message>) {
        (
            MyApp {
                username: "Test1".to_string(),
            },
            Command::none(),
        )
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
        Command::none()
    }
}

fn main() {
    MyApp::run()
}
