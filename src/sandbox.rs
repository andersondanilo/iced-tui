use crate::TuiRenderer;
use core::cell::RefCell;
use iced_core::Point;
use iced_core::Size;
use iced_native::Command;
use iced_native::Executor;
use iced_native::Hasher;
use iced_native::Subscription;
use iced_native::{Cache, Container, Element, Length, Renderer, UserInterface};
use std::rc::Rc;
use tui::backend::Backend;
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
        use iced_futures::futures::stream::StreamExt;

        let (sender, receiver) = iced_futures::futures::channel::mpsc::unbounded();

        let mut runtime = iced_futures::Runtime::new(
            Self::Executor::new().expect("Create executor"),
            sender.clone(),
        );

        let (app, command) = runtime.enter(|| Self::new());

        runtime.spawn(command);

        // TODO: Figure out how keyboard events are processed

        let application = Rc::new(RefCell::new(app));
        let mut renderer = TuiRenderer::default();

        let event_loop = receiver.for_each(move |message| {
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

            let (command, subscription) = runtime.enter(|| {
                let command = application.borrow_mut().update(message);
                let subscription = application.borrow().subscription();

                (command, subscription)
            });

            runtime.spawn(command);
            runtime.track(subscription);

            iced_futures::futures::future::ready(())
        });

        // TODO: Spawn loop, as iced_web
        // Creates the sandbox and its renderer
        //let mut state = Self::new();

        //let mut cache = Some(Cache::default());

        //loop {
        //    let renderer = TuiRenderer::default();
        //    let tui_size = terminal.size().unwrap();
        //    let cursor_position = Point { x: 0.0, y: 0.0 };

        //    let size = Size {
        //        width: tui_size.width as f32,
        //        height: tui_size.height as f32,
        //    };

        //    // Consumes the cache and renders the UI to primitives
        //    let view: Element<'_, Self::Message, TuiRenderer> = Container::new(state.view())
        //        .width(Length::Units(tui_size.width))
        //        .height(Length::Units(tui_size.height))
        //        .into();

        //    let mut ui = UserInterface::build(view, size, cache.take().unwrap(), &mut renderer);

        //    // Displays the new state of the sandbox using the renderer
        //    let primitives = ui.draw(&mut renderer, cursor_position);
        //    renderer.draw(primitives);

        //    // Polls pancurses events and apply them on the ui
        //    let messages = renderer
        //        .handle()
        //        .map(|events| ui.update(&renderer, None, events.into_iter()));

        //    // Stores back the cache
        //    cache = some(ui.into_cache());

        //    // Applies updates on the state with given messages if any
        //    if let Some(messages) = messages {
        //        state.update(messages);
        //    }
        //}
    }
}
