use crate::constants::LOG_TARGET;
use crate::renderer::VirtualBuffer;
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
use iced_native::mouse;
use iced_native::window;
use iced_native::Command;
use iced_native::Event;
use iced_native::Executor;
use iced_native::Subscription;
use iced_native::UserInterface;
use iced_native::{Cache, Element};
use std::rc::Rc;
use std::sync::atomic::AtomicU16;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

pub trait Application {
    type Executor: Executor;
    type Message: Send + Clone;

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

    /// Returns whether the [`Application`] should be terminated (and the exit status code).
    ///
    /// By default, it returns None.
    fn should_exit(&self) -> Option<u8> {
        None
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

        let last_mouse_position = Arc::new((AtomicU16::new(0), AtomicU16::new(0)));

        // ui iced-events loop
        //
        let ui_iced_event_sender = sender.clone();
        let last_mouse_position_writer = last_mouse_position.clone();
        runtime_executor.spawn(async move {
            loop {
                let event = event::read().unwrap();

                let iced_events = match event {
                    event::Event::Key(key_event) => {
                        map_keycode_event(key_event.code, key_event.modifiers)
                            .into_iter()
                            .map(Event::Keyboard)
                            .collect()
                    }
                    event::Event::Mouse(mouse_event) => {
                        last_mouse_position_writer
                            .0
                            .store(mouse_event.column, Ordering::Relaxed);

                        last_mouse_position_writer
                            .1
                            .store(mouse_event.row, Ordering::Relaxed);

                        map_mouse_event(mouse_event)
                            .into_iter()
                            .map(Event::Mouse)
                            .collect()
                    }
                    event::Event::Resize(width, height) => {
                        vec![Event::Window(window::Event::Resized {
                            width: width as u32,
                            height: height as u32,
                        })]
                    }
                };

                if let Err(err) =
                    ui_iced_event_sender.unbounded_send(UiMessage::from_events(iced_events))
                {
                    log::error!(target: LOG_TARGET, "{}", err);
                    return;
                }
            }
        });

        let mapper = AppMessageMapperSink::from_sender(sender);
        let mut runtime = iced_futures::Runtime::new(runtime_executor, mapper);

        let (app, command) = runtime.enter(Self::new);
        let application = Rc::new(RefCell::new(app));

        runtime.spawn(command);

        let mut cache = Some(Cache::default());
        let mut renderer = TuiRenderer::default();
        let mut last_vbuffer: Option<VirtualBuffer> = None;

        let mut stdout = std::io::stdout();
        renderer.begin_screen(&mut stdout);

        let mut ui_message: Option<UiMessage<Self::Message>> = None;

        // event loop on main thread
        let exit_status_code: u8 = loop {
            // TODO: Review logic about immeditate render when state is updated
            let mut state_updated = false;
            let current_ui_message = ui_message.clone();
            let (commands, subscription, events, event_statuses) = runtime.enter(|| {
                let (width, height) = terminal::size().unwrap();
                let size = Size {
                    width: width as f32,
                    height: height as f32,
                };

                let cursor_position = Point {
                    x: (&last_mouse_position.0.load(Ordering::Relaxed)).clone() as f32,
                    y: (&last_mouse_position.1.load(Ordering::Relaxed)).clone() as f32,
                };

                //log::debug!(target: LOG_TARGET, "received message");

                // render and return messages
                let mut app_bmut = application.borrow_mut();
                let view_result = app_bmut.view();

                let mut ui =
                    UserInterface::build(view_result, size, cache.take().unwrap(), &mut renderer);
                let primitive = ui.draw(&mut renderer, cursor_position);

                if let Some(vbuffer) = renderer.render(&mut stdout, primitive, &last_vbuffer) {
                    last_vbuffer = Some(vbuffer);
                }

                let (messages, event_statuses, events, ui_updated) = match current_ui_message {
                    Some(ui_message) => {
                        let mut messages: Vec<Self::Message> = match ui_message.app_message {
                            Some(m) => vec![m],
                            None => vec![],
                        };
                        let mut event_statuses = vec![];
                        let events = ui_message.events.clone();
                        let mut ui_updated = false;

                        if !ui_message.events.is_empty() {
                            event_statuses = ui.update(
                                &ui_message.events,
                                cursor_position,
                                &renderer,
                                &mut clipboard::Null,
                                &mut messages,
                            );
                            ui_updated = true;
                        }

                        (messages, event_statuses, events, ui_updated)
                    }
                    None => (vec![], vec![], vec![], false),
                };

                cache = Some(ui.into_cache());

                // update state
                let mut commands: Vec<Command<Self::Message>> = vec![];

                for message in messages {
                    commands.push(app_bmut.update(message));
                    state_updated = true;
                }

                if ui_updated {
                    state_updated = true;
                }

                let subscription = app_bmut.subscription();

                (commands, subscription, events, event_statuses)
            });

            for command in commands {
                runtime.spawn(command);
            }

            for (event, event_status) in events.iter().zip(event_statuses) {
                //log::debug!(
                //    target: LOG_TARGET,
                //    "broadcasting event: {:?}, status: {:?}",
                //    event,
                //    event_status
                //);
                runtime.broadcast((event.clone(), event_status));
            }

            runtime.track(subscription);

            if let Some(status_code) = application.borrow().should_exit() {
                //log::debug!(target: LOG_TARGET, "Exiting app");
                break status_code;
            }

            ui_message = loop {
                match receiver.try_next() {
                    Ok(Some(m)) => break Some(m),
                    Ok(None) => {
                        log::error!(
                            target: LOG_TARGET,
                            "{:?} - Channel closed",
                            std::thread::current().id()
                        );
                        std::thread::sleep(poll_rate);
                        continue;
                    }
                    Err(_) => {
                        if !state_updated {
                            std::thread::sleep(poll_rate);
                        } else {
                            break Some(UiMessage {
                                app_message: None,
                                events: vec![],
                            });
                        }
                    }
                }
            };
        };

        renderer.end_screen(&mut stdout);

        std::process::exit(exit_status_code.into());
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
        event::KeyCode::F(number) => match number {
            1 => Some(keyboard::KeyCode::F1),
            2 => Some(keyboard::KeyCode::F2),
            3 => Some(keyboard::KeyCode::F3),
            4 => Some(keyboard::KeyCode::F4),
            5 => Some(keyboard::KeyCode::F5),
            6 => Some(keyboard::KeyCode::F6),
            7 => Some(keyboard::KeyCode::F7),
            8 => Some(keyboard::KeyCode::F8),
            9 => Some(keyboard::KeyCode::F9),
            10 => Some(keyboard::KeyCode::F10),
            11 => Some(keyboard::KeyCode::F11),
            12 => Some(keyboard::KeyCode::F12),
            _ => None,
        },
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

fn map_mouse_event(mouse_event: event::MouseEvent) -> Vec<mouse::Event> {
    match mouse_event.kind {
        event::MouseEventKind::Down(button) => {
            vec![mouse::Event::ButtonPressed(map_mouse_button(button))]
        }
        event::MouseEventKind::Up(button) => {
            vec![mouse::Event::ButtonReleased(map_mouse_button(button))]
        }
        event::MouseEventKind::Drag(_) => vec![],
        event::MouseEventKind::Moved => vec![mouse::Event::CursorMoved {
            position: Point::new(mouse_event.column as f32, mouse_event.row as f32),
        }],
        event::MouseEventKind::ScrollDown => vec![mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Lines { x: 0_f32, y: 1_f32 },
        }],
        event::MouseEventKind::ScrollUp => vec![mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Lines { x: 0_f32, y: 1_f32 },
        }],
    }
}

fn map_mouse_button(button: event::MouseButton) -> mouse::Button {
    match button {
        event::MouseButton::Left => mouse::Button::Left,
        event::MouseButton::Right => mouse::Button::Right,
        event::MouseButton::Middle => mouse::Button::Middle,
    }
}

#[derive(Debug, Clone)]
struct UiMessage<M> {
    app_message: Option<M>,
    events: Vec<Event>,
}

impl<M> UiMessage<M> {
    fn from_events(events: Vec<Event>) -> Self {
        UiMessage {
            app_message: None,
            events,
        }
    }

    fn from_app_message(app_message: M) -> Self {
        UiMessage {
            app_message: Some(app_message),
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
        Self { sender }
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
        self.get_mut()
            .sender
            .start_send(UiMessage::from_app_message(message))
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), <Self as futures::Sink<M>>::Error>> {
        std::pin::Pin::new(&mut self.get_mut().sender).poll_flush(ctx)
    }

    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), <Self as iced_futures::futures::Sink<M>>::Error>>
    {
        std::pin::Pin::new(&mut self.get_mut().sender).poll_close(ctx)
    }
}
