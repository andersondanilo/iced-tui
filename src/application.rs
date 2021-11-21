use crate::TuiRenderer;
use core::cell::RefCell;
pub use crossterm::{
    cursor,
    event::{self, KeyCode, KeyEvent},
    execute, queue, style,
    terminal::{self, ClearType},
    Result,
};
use iced_core::Point;
use iced_core::Size;
use iced_futures::futures::{self, channel::mpsc, Sink};
use iced_native::clipboard;
use iced_native::keyboard;
use iced_native::Command;
use iced_native::Event;
use iced_native::Executor;
use iced_native::Hasher;
use iced_native::Subscription;
use iced_native::UserInterface;
use iced_native::{Cache, Container, Element, Length};
use std::rc::Rc;
use std::time::Duration;

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
    fn update(&mut self, message: Self::Message) -> Command<Self::Message>;

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
    fn run()
    where
        Self: 'static + Sized + Send,
        <Self as Application>::Executor: Send,
    {
        let (sender, mut receiver) = mpsc::unbounded::<UiMessage<Self::Message>>();

        let runtime_executor = Self::Executor::new().expect("Create executor");

        let poll_rate = Duration::from_millis(15);

        let orig_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            // invoke the default handler and exit the process
            orig_hook(panic_info);
            std::process::exit(1);
        }));

        // ui iced-events loop
        let ui_iced_event_sender = sender.clone();
        runtime_executor.spawn(async move {
            loop {
                let event = event::read().unwrap();

                let term_events = match event {
                    event::Event::Key(key_event) => {
                        map_keycode_event(key_event.code, key_event.modifiers)
                    }
                    _ => vec![],
                };

                eprintln!("trying to send key event");

                if let Err(err) = ui_iced_event_sender.unbounded_send(UiMessage::from_events(
                    term_events
                        .into_iter()
                        .map(|k| Event::Keyboard(k))
                        .collect(),
                )) {
                    eprintln!("{}", err);
                    return;
                }
            }
        });

        let mut runtime = iced_futures::Runtime::new(
            runtime_executor,
            AppMessageMapperSink::from_sender(sender.clone()),
        );

        let (app, command) = runtime.enter(|| Self::new());
        let application = Rc::new(RefCell::new(app));

        runtime.spawn(command);

        let mut cache = Some(Cache::default());
        let mut renderer = TuiRenderer::default();
        let mut has_first_render = false;

        let mut stdout = std::io::stdout();
        renderer.begin_screen(&mut stdout);

        // event loop on main thread
        loop {
            // eprintln!("{:?} - Waiting next message", std::thread::current().id());
            if application.borrow().should_exit() {
                eprintln!("Exiting app");
                break;
            }

            let ui_message = match receiver.try_next() {
                Ok(Some(m)) => m,
                Ok(None) => {
                    eprintln!("{:?} - Channel closed", std::thread::current().id());
                    std::thread::sleep(poll_rate);
                    continue;
                }
                Err(_) => {
                    if has_first_render {
                        // eprintln!("{:?} - No next message", e);
                        std::thread::sleep(poll_rate);
                        continue;
                    } else {
                        has_first_render = true;
                        UiMessage {
                            app_message: None,
                            events: vec![],
                        }
                    }
                }
            };

            let (commands, subscription, events) = runtime.enter(|| {
                eprintln!(
                    "{:?} - Received: {:?}",
                    std::thread::current().id(),
                    ui_message,
                );
                let (width, height) = terminal::size().unwrap();
                let size = Size {
                    width: width as f32,
                    height: height as f32,
                };
                let cursor_position = Point { x: 0.0, y: 0.0 };
                let events = ui_message.events.clone();

                eprintln!("Rendering on size: {:?}", size);

                // render and return messages
                let messages = {
                    let mut app_bmut = application.borrow_mut();
                    let view_result = app_bmut.view();

                    let mut ui = UserInterface::build(
                        view_result,
                        size,
                        cache.take().unwrap(),
                        &mut renderer,
                    );
                    let primitive = ui.draw(&mut renderer, cursor_position);

                    renderer.render(&mut stdout, primitive);

                    let mut messages: Vec<Self::Message> = match ui_message.app_message {
                        Some(m) => vec![m],
                        None => vec![],
                    };

                    eprintln!("messages received from loop : {:?}", messages);

                    if !ui_message.events.is_empty() {
                        ui.update(
                            &ui_message.events,
                            cursor_position,
                            &renderer,
                            &mut clipboard::Null,
                            &mut messages,
                        );
                        eprintln!("messages after events: {:?}", messages);
                    }

                    cache = Some(ui.into_cache());

                    messages
                };

                eprintln!("current messages: {:?}", messages);

                // update state
                let mut commands: Vec<Command<Self::Message>> = vec![];
                let mut app_bmut = application.borrow_mut();

                for message in messages {
                    commands.push(app_bmut.update(message));
                }

                eprintln!("current commands: {:?}", commands);

                let subscription = app_bmut.subscription();

                (commands, subscription, events)
            });

            for command in commands {
                runtime.spawn(command);
            }

            for event in events {
                eprintln!("broadcasting event: {:?}", event);
                runtime.broadcast((event, iced_native::event::Status::Ignored));
            }

            eprintln!("tracking subscription: {:?}", subscription);
            runtime.track(subscription);
        }

        renderer.end_screen(&mut stdout);
        std::process::exit(0);
    }
}

fn map_keycode_event(
    term_keycode: event::KeyCode,
    term_keymod: event::KeyModifiers,
) -> Vec<keyboard::Event> {
    let iced_keycode = term_keycode_to_iced(term_keycode);

    let modifiers = term_keymod_to_iced(term_keymod);

    let mut events: Vec<keyboard::Event> = match iced_keycode {
        Some(key_code) => vec![keyboard::Event::KeyPressed {
            key_code,
            modifiers,
        }],
        None => vec![],
    };

    if !modifiers.control {
        if let event::KeyCode::Char(c) = term_keycode {
            events.push(keyboard::Event::CharacterReceived(c));
        }
    }

    if let Some(keycode) = iced_keycode {
        events.push(keyboard::Event::KeyReleased {
            key_code: keycode,
            modifiers: term_keymod_to_iced(term_keymod),
        });
    }

    events
}

fn term_keycode_to_iced(term_keycode: event::KeyCode) -> Option<keyboard::KeyCode> {
    match term_keycode {
        event::KeyCode::Backspace => Some(keyboard::KeyCode::Backspace),
        event::KeyCode::Enter => Some(keyboard::KeyCode::Enter),
        event::KeyCode::Left => Some(keyboard::KeyCode::Left),
        event::KeyCode::Right => Some(keyboard::KeyCode::Right),
        event::KeyCode::Up => Some(keyboard::KeyCode::Up),
        event::KeyCode::Down => Some(keyboard::KeyCode::Down),
        event::KeyCode::Home => Some(keyboard::KeyCode::Home),
        event::KeyCode::End => Some(keyboard::KeyCode::End),
        event::KeyCode::PageUp => Some(keyboard::KeyCode::PageUp),
        event::KeyCode::PageDown => Some(keyboard::KeyCode::PageDown),
        event::KeyCode::Tab => Some(keyboard::KeyCode::Tab),
        event::KeyCode::BackTab => Some(keyboard::KeyCode::Tab),
        event::KeyCode::Delete => Some(keyboard::KeyCode::Delete),
        event::KeyCode::Insert => Some(keyboard::KeyCode::Insert),
        event::KeyCode::F(u8) => None, // TODO: Map F* keys
        event::KeyCode::Char(c) => keycode_from_char(c),
        event::KeyCode::Null => None,
        event::KeyCode::Esc => Some(keyboard::KeyCode::Escape),
    }
}

fn term_keymod_to_iced(term_keymod: event::KeyModifiers) -> keyboard::Modifiers {
    keyboard::Modifiers {
        alt: term_keymod.contains(event::KeyModifiers::ALT),
        control: term_keymod.contains(event::KeyModifiers::CONTROL),
        logo: false,
        shift: term_keymod.contains(event::KeyModifiers::SHIFT),
    }
}

fn keycode_from_char(c: char) -> Option<keyboard::KeyCode> {
    match c.to_uppercase().next() {
        Some('A') => Some(keyboard::KeyCode::A),
        Some('B') => Some(keyboard::KeyCode::B),
        Some('C') => Some(keyboard::KeyCode::C),
        Some('D') => Some(keyboard::KeyCode::D),
        Some('E') => Some(keyboard::KeyCode::E),
        Some('F') => Some(keyboard::KeyCode::F),
        Some('G') => Some(keyboard::KeyCode::G),
        Some('H') => Some(keyboard::KeyCode::H),
        Some('I') => Some(keyboard::KeyCode::I),
        Some('J') => Some(keyboard::KeyCode::J),
        Some('K') => Some(keyboard::KeyCode::K),
        Some('L') => Some(keyboard::KeyCode::L),
        Some('M') => Some(keyboard::KeyCode::M),
        Some('N') => Some(keyboard::KeyCode::N),
        Some('O') => Some(keyboard::KeyCode::O),
        Some('P') => Some(keyboard::KeyCode::P),
        Some('Q') => Some(keyboard::KeyCode::Q),
        Some('R') => Some(keyboard::KeyCode::R),
        Some('S') => Some(keyboard::KeyCode::S),
        Some('T') => Some(keyboard::KeyCode::T),
        Some('U') => Some(keyboard::KeyCode::U),
        Some('V') => Some(keyboard::KeyCode::V),
        Some('X') => Some(keyboard::KeyCode::X),
        Some('Y') => Some(keyboard::KeyCode::Y),
        Some('Z') => Some(keyboard::KeyCode::Z),
        _ => None,
    }
}

#[derive(Debug)]
struct UiMessage<M> {
    app_message: Option<M>,
    events: Vec<Event>,
}

impl<M> UiMessage<M> {
    fn from_events(events: Vec<Event>) -> Self {
        UiMessage {
            app_message: None,
            events: events,
        }
    }

    fn from_app_message(app_message: M) -> Self {
        UiMessage {
            app_message: Some(app_message),
            events: vec![],
        }
    }

    fn tick() -> Self {
        UiMessage {
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

impl<M> Sink<M> for AppMessageMapperSink<M>
where
    M: std::fmt::Debug,
{
    type Error = mpsc::SendError;

    fn poll_ready(
        self: std::pin::Pin<&mut Self>,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), <Self as futures::Sink<M>>::Error>> {
        eprintln!("call poll ready");
        self.sender.poll_ready(ctx)
    }

    fn start_send(
        self: std::pin::Pin<&mut Self>,
        message: M,
    ) -> std::result::Result<(), <Self as futures::Sink<M>>::Error> {
        eprintln!("start sink send: {:?}", message);
        self.get_mut()
            .sender
            .start_send(UiMessage::from_app_message(message))
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), <Self as futures::Sink<M>>::Error>> {
        eprintln!("call poll flush");
        std::pin::Pin::new(&mut self.get_mut().sender).poll_flush(ctx)
    }

    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), <Self as iced_futures::futures::Sink<M>>::Error>>
    {
        eprintln!("call poll close");
        std::pin::Pin::new(&mut self.get_mut().sender).poll_close(ctx)
    }
}