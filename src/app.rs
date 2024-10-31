use std::sync::mpsc::Sender;

use tracing::error;
use uuid::Uuid;
use windows::Win32::UI::WindowsAndMessaging::MB_ICONERROR;
use winit::{application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop};

use crate::{active_app::ActiveApp, message::Message, message_box::message_box};

/// The core app
pub struct App {
    /// The initialised app.
    pub active_app: Option<ActiveApp>,

    /// The current break's ID.
    pub break_id: Option<Uuid>,

    /// The sender to contant the notifer.
    pub message_sender: Sender<Message>,
}

impl App {
    pub fn new(message_sender: Sender<Message>) -> Self {
        Self {
            active_app: None,
            break_id: None,
            message_sender,
        }
    }

    /// Triggers the end of the break to reset the timer
    pub fn finish_break(&mut self) {
        // send end notification
        if let Some(break_id) = self.break_id.take() {
            if let Err(e) = self.message_sender.send(Message::EndBreak(break_id)) {
                error!("Failed to send break end to waker thread:\n{e}");
                message_box("Failed to send break end to waker thread.", MB_ICONERROR);
                panic!("Failed to send break end to waker thread:\n{e}");
            }
        }
    }

    /// Tries to show the break reminder, if app isn't active then the break is ended.
    pub fn show_break_reminder(&mut self, id: Uuid) {
        let Some(app) = self.active_app.as_ref() else {
            self.finish_break();
            return;
        };

        app.move_window_to_best_monitor();
        self.break_id = Some(id);
        app.show_break_reminder(); // blocking
        self.finish_break();
    }
}

impl ApplicationHandler<Uuid> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        match ActiveApp::new(event_loop) {
            Ok(app) => self.active_app = Some(app),
            Err(e) => {
                error!("Failed to initialize app:\n{e}");
                message_box("Failed to initialize app.", MB_ICONERROR);
                panic!("Failed to initialize app:\n{e}");
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let Some(app) = self.active_app.as_ref() else {
            return;
        };

        if event == WindowEvent::Destroyed && app.window.id() == window_id {
            self.active_app = None;
            event_loop.exit();
            return;
        }

        #[allow(clippy::single_match)]
        match event {
            WindowEvent::CloseRequested => {
                self.active_app = None;
                event_loop.exit();
            }

            _ => {}
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: Uuid) {
        self.show_break_reminder(event);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let Some(app) = self.active_app.as_ref() else {
            return;
        };

        if let Some(message) = app.handle_tray_icon(event_loop) {
            if let Err(e) = self.message_sender.send(message) {
                error!("Failed to send break end to waker thread:\n{e}");
                message_box("Failed to send break end to waker thread.", MB_ICONERROR);
                panic!("Failed to send break end to waker thread:\n{e}");
            };
        };
    }
}
