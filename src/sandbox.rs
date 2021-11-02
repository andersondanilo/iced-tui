use crate::TuiRenderer;
use iced_core::Point;
use iced_core::Size;
use iced_native::Command;
use iced_native::Executor;
use iced_native::Subscription;
use iced_native::{Cache, Container, Element, Length, Renderer, UserInterface};
use tui::backend::Backend;
use tui::Terminal;

pub trait Application: Sized {
    type Executor: Executor;
    type Message: std::fmt::Debug + Send + Clone;

    /// Initializes the Sanbox
    ///
    /// Should return the initial state of the sandbox
    fn new() -> Self;

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
        Self: 'static,
    {
        // Creates the sandbox and its renderer
        let mut state = Self::new();

        let mut cache = Some(Cache::default());

        loop {
            let renderer = TuiRenderer::default();
            let tui_size = terminal.size().unwrap();
            let cursor_position = Point { x: 0.0, y: 0.0 };

            let size = Size {
                width: tui_size.width as f32,
                height: tui_size.height as f32,
            };

            // Consumes the cache and renders the UI to primitives
            let view: Element<'_, Self::Message, TuiRenderer> = Container::new(state.view())
                .width(Length::Units(tui_size.width))
                .height(Length::Units(tui_size.height))
                .into();

            let mut ui = UserInterface::build(view, size, cache.take().unwrap(), &mut renderer);

            // Displays the new state of the sandbox using the renderer
            let primitives = ui.draw(&mut renderer, cursor_position);
            renderer.draw(primitives);

            // Polls pancurses events and apply them on the ui
            let messages = renderer
                .handle()
                .map(|events| ui.update(&renderer, None, events.into_iter()));

            // Stores back the cache
            cache = Some(ui.into_cache());

            // Applies updates on the state with given messages if any
            if let Some(messages) = messages {
                state.update(messages);
            }
        }
    }
}
