use crate::TuiRenderer;
use core::cell::RefCell;
use futures::channel::mpsc;
use futures::Sink;
use iced_core::Point;
use iced_core::Size;
use iced_native::clipboard;
use iced_native::keyboard;
use iced_native::Command;
use iced_native::Event;
use iced_native::Executor;
use iced_native::Hasher;
use iced_native::Subscription;
use iced_native::UserInterface;
use iced_native::{Cache, Container, Element, Length, Renderer};
use std::rc::Rc;
use std::time::Duration;
use tui::backend::Backend;
// TODO: decouple from termion here (function keys in stdin)
use termion::event::Key;
use termion::input::TermRead;

use tui::Terminal;

pub trait Application {
    type Executor: Executor;
    type Message: std::fmt::Debug + Send + Clone;

    /// Initializes the Sanbox
    ///
    /// Should return the initial state of the sandbox
    fn new() -> (Self, Command<Self::Message>)
    where
        Self: Sized;

    /// Handles a __message__ and updates the state of the [`Application`].
    ///
    /// This is where you define your __update logic__. All the __messages__,
    /// produced by either user interactions or commands, will be handled by
    /// this method.
    ///
    /// Any [`Command`] returned will be executed immediately in the background.
    fn update(&mut self, messages: Self::Message) -> Command<Self::Message>;

    /// Returns the widgets to display in the [`Application`].
    ///
    /// These widgets can produce __messages__ based on user interaction.
    fn view(&mut self) -> Element<'_, Self::Message, TuiRenderer>;

    /// Returns the event [`Subscription`] for the current state of the
    /// application.
    ///
    /// A [`Subscription`] will be kept alive as long as you keep returning it,
    /// and the __messages__ produced will be handled by
    /// [`update`](#tymethod.update).
    ///
    /// By default, this method returns an empty [`Subscription`].
    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }

    /// Returns whether the [`Application`] should be terminated.
    ///
    /// By default, it returns `false`.
    fn should_exit(&self) -> bool {
        false
    }

    /// Launches the sandbox and takes ownership of the current thread.
    ///
    /// This should be the last thing you execute at the end of the entrypoint of
    /// your program.
    fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>)
    where
        Self: 'static + Sized,
    {
        use futures::stream::StreamExt;

        let (sender, receiver) = mpsc::unbounded::<UiMessage<Self::Message>>();

        let executor = Self::Executor::new().expect("Create executor");

        let tick_rate = Duration::from_millis(250);

        let mut runtime =
            iced_futures::Runtime::new(executor, AppMessageMapperSink::from_sender(sender.clone()));

        let (app, command) = runtime.enter(|| Self::new());

        runtime.spawn(command);

        let application = Rc::new(RefCell::new(app));
        let mut renderer = TuiRenderer::default();
        let mut cache = Some(Cache::default());

        let ui_event_loop = receiver.for_each(move |ui_message| {
            let tui_size = terminal.size().unwrap();

            let size = Size {
                width: tui_size.width as f32,
                height: tui_size.height as f32,
            };

            let mut application_borrow_mut = application.borrow_mut();
            let view_result = application_borrow_mut.view();
            let cursor_position = Point { x: 0.0, y: 0.0 };

            let view: Element<'_, Self::Message, TuiRenderer> = Container::new(view_result)
                .width(Length::Units(tui_size.width))
                .height(Length::Units(tui_size.height))
                .into();

            let mut ui = UserInterface::build(view, size, cache.take().unwrap(), &mut renderer);
            let primitives = ui.draw(&mut renderer, cursor_position);

            renderer.render(&terminal, primitives);

            let (commands, subscription) = runtime.enter(|| {
                let mut app_bmut = application.borrow_mut();

                let commands = match ui_message.app_message {
                    Some(m) => vec![app_bmut.update(m)],
                    None => vec![],
                };

                if !ui_message.events.is_empty() {
                    let mut event_messages = vec![];

                    ui.update(
                        &ui_message.events,
                        cursor_position,
                        &renderer,
                        &mut clipboard::Null,
                        &mut event_messages,
                    );

                    for event_message in event_messages {
                        commands.push(app_bmut.update(event_message));
                    }
                }

                let subscription = application.borrow().subscription();

                (commands, subscription)
            });

            for command in commands {
                runtime.spawn(command);
            }
            runtime.track(subscription);

            futures::future::ready(())
        });

        executor.spawn(ui_event_loop);

        // ui events loop
        let ui_event_sender = sender.clone();
        executor.spawn(async {
            let stdin = std::io::stdin();
            for evt in stdin.keys() {
                if let Ok(key) = evt {
                    let kb_events = match key {
                        Key::Backspace => {
                            key_pressed_without_modifiers(keyboard::KeyCode::Backspace)
                        }
                        Key::Left => key_pressed_without_modifiers(keyboard::KeyCode::Left),
                        Key::Right => key_pressed_without_modifiers(keyboard::KeyCode::Right),
                        Key::Up => key_pressed_without_modifiers(keyboard::KeyCode::Up),
                        Key::Down => key_pressed_without_modifiers(keyboard::KeyCode::Down),
                        Key::Home => key_pressed_without_modifiers(keyboard::KeyCode::Home),
                        Key::End => key_pressed_without_modifiers(keyboard::KeyCode::End),
                        Key::PageUp => key_pressed_without_modifiers(keyboard::KeyCode::PageUp),
                        Key::PageDown => key_pressed_without_modifiers(keyboard::KeyCode::PageDown),
                        //Key::BackTab => {
                        //    key_pressed_without_modifiers(keyboard::KeyCode::Tab)]
                        //},
                        Key::Delete => key_pressed_without_modifiers(keyboard::KeyCode::Delete),
                        Key::Insert => key_pressed_without_modifiers(keyboard::KeyCode::Insert),
                        //Key::F(n) => {
                        //    key_pressed_without_modifiers(keyboard::KeyCode::F1)]
                        //},
                        Key::Char(c) => char_received(c),
                        Key::Alt(c) => char_received_with_modifiers(
                            c,
                            keyboard::Modifiers {
                                alt: true,
                                control: false,
                                logo: false,
                                shift: false,
                            },
                        ),
                        Key::Ctrl(c) => char_received_with_modifiers(
                            c,
                            keyboard::Modifiers {
                                alt: false,
                                control: true,
                                logo: false,
                                shift: false,
                            },
                        ),
                        //Key::Null,
                        Key::Esc => key_pressed_without_modifiers(keyboard::KeyCode::Escape),
                    };

                    if let Err(err) = ui_event_sender.unbounded_send(UiMessage::from_events(
                        kb_events.into_iter().map(|k| Event::Keyboard(k)).collect(),
                    )) {
                        eprintln!("{}", err);
                        return;
                    }
                }
            }
        });

        // ui tick loop
        let ui_tick_sender = sender.clone();
        executor.spawn(async {
            loop {
                if let Err(err) = ui_event_sender.unbounded_send(UiMessage::tick()) {
                    eprintln!("{}", err);
                    break;
                }
                std::thread::sleep(tick_rate);
            }
        });
    }
}

fn key_pressed_without_modifiers(keycode: keyboard::KeyCode) -> Vec<keyboard::Event> {
    vec![
        keyboard::Event::KeyPressed {
            key_code: keycode,
            modifiers: keyboard::Modifiers {
                alt: false,
                control: false,
                logo: false,
                shift: false,
            },
        },
        keyboard::Event::KeyReleased {
            key_code: keycode,
            modifiers: keyboard::Modifiers {
                alt: false,
                control: false,
                logo: false,
                shift: false,
            },
        },
    ]
}

fn char_received(c: char) -> Vec<keyboard::Event> {
    char_received_with_modifiers(
        c,
        keyboard::Modifiers {
            alt: false,
            control: false,
            logo: false,
            shift: false,
        },
    )
}

fn char_received_with_modifiers(c: char, modifiers: keyboard::Modifiers) -> Vec<keyboard::Event> {
    let mut events = vec![];
    let keycode = keycode_from_char(c);

    if let Some(key) = keycode {
        events.push(keyboard::Event::KeyPressed {
            key_code: key,
            modifiers,
        });
    }

    if !modifiers.alt && !modifiers.control {
        events.push(keyboard::Event::CharacterReceived(c));
    }

    if let Some(key) = keycode {
        events.push(keyboard::Event::KeyReleased {
            key_code: key,
            modifiers,
        });
    }

    events
}

fn keycode_from_char(c: char) -> Option<keyboard::KeyCode> {
    c.to_uppercase().next().map(|c| match c {
        'A' => keyboard::KeyCode::A,
        'B' => keyboard::KeyCode::B,
        'C' => keyboard::KeyCode::C,
        'D' => keyboard::KeyCode::D,
        'E' => keyboard::KeyCode::E,
        'F' => keyboard::KeyCode::F,
        'G' => keyboard::KeyCode::G,
        'H' => keyboard::KeyCode::H,
        'I' => keyboard::KeyCode::I,
        'J' => keyboard::KeyCode::J,
        'K' => keyboard::KeyCode::K,
        'L' => keyboard::KeyCode::L,
        'M' => keyboard::KeyCode::M,
        'N' => keyboard::KeyCode::N,
        'O' => keyboard::KeyCode::O,
        'P' => keyboard::KeyCode::P,
        'Q' => keyboard::KeyCode::Q,
        'R' => keyboard::KeyCode::R,
        'S' => keyboard::KeyCode::S,
        'T' => keyboard::KeyCode::T,
        'U' => keyboard::KeyCode::U,
        'V' => keyboard::KeyCode::V,
        'X' => keyboard::KeyCode::X,
        'Y' => keyboard::KeyCode::Y,
        'Z' => keyboard::KeyCode::Z,
    })
}

enum UiMessageType {
    AppMessage,
    UiEvent,
    UiTick,
}

struct UiMessage<M> {
    message_type: UiMessageType,
    app_message: Option<M>,
    events: Vec<Event>,
}

impl<M> UiMessage<M> {
    fn from_events(events: Vec<Event>) -> Self {
        UiMessage {
            message_type: UiMessageType::AppMessage,
            app_message: None,
            events: events,
        }
    }

    fn from_app_message(app_message: M) -> Self {
        UiMessage {
            message_type: UiMessageType::AppMessage,
            app_message: Some(app_message),
            events: vec![],
        }
    }

    fn tick() -> Self {
        UiMessage {
            message_type: UiMessageType::UiTick,
            app_message: None,
            events: vec![],
        }
    }
}

#[derive(Clone)]
struct AppMessageMapperSink<M> {
    sender: mpsc::UnboundedSender<UiMessage<M>>,
}

impl<M> AppMessageMapperSink<M> {
    fn from_sender(sender: mpsc::UnboundedSender<UiMessage<M>>) -> Self {
        Self { sender: sender }
    }
}

impl<M> Sink<M> for AppMessageMapperSink<M> {
    type Error = mpsc::SendError;

    fn poll_ready(
        self: std::pin::Pin<&mut Self>,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), <Self as futures::Sink<M>>::Error>> {
        self.sender.poll_ready(ctx)
    }

    fn start_send(
        self: std::pin::Pin<&mut Self>,
        message: M,
    ) -> std::result::Result<(), <Self as futures::Sink<M>>::Error> {
        self.sender.start_send(UiMessage::from_app_message(message))
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), <Self as futures::Sink<M>>::Error>> {
        std::pin::Pin::new(&mut self.sender).poll_flush(ctx)
    }

    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), <Self as iced_futures::futures::Sink<M>>::Error>>
    {
        std::pin::Pin::new(&mut self.sender).poll_close(ctx)
    }
}
