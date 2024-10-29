#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::sync::mpsc::channel;

use app::App;
use logger::init_tracing;
use message_box::message_box;
use notifier::start_notifier;
use only_instance::is_only_instance;
use thiserror::Error;
use tracing::{error, warn};
use uuid::Uuid;
use windows::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_ICONWARNING};
use winit::{error::EventLoopError, event_loop::EventLoop};

mod app;
mod logger;
pub mod message_box;
mod notifier;
mod only_instance;

/// App to send a message box with a given interval after the previous message box has been interacted with.
/// The interval is specified in minutes by the first command line argument and defaults to 20 minutes.
fn main() {
    let _log_guards = match init_tracing() {
        Ok(guards) => guards,
        Err(e) => {
            message_box(format!("Failed to init tracing. {e}"), MB_ICONERROR);
            panic!("Failed to init tracing:\n{e}");
        }
    };

    if let Err(e) = start_app() {
        error!("Failed to start app:\n{e}");
        message_box("Failed to start app.", MB_ICONERROR);
        panic!("Failed to start app:\n{e}");
    }
}

fn start_app() -> Result<(), Error> {
    if !is_only_instance()? {
        warn!("Another instance is already running");
        message_box("Another instance is already running.", MB_ICONWARNING);
        return Ok(());
    };

    // Create event loop
    let event_loop: EventLoop<Uuid> = EventLoop::with_user_event().build()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
    let (break_end_sender, break_end_receiver) = channel::<Uuid>();

    let proxy = event_loop.create_proxy();
    start_notifier(proxy, break_end_receiver);

    // Create the app
    let mut app = App::new(break_end_sender);

    // run the app
    event_loop.run_app(&mut app)?;

    Ok(())
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("Failed to ensure only one instance:\n{0}")]
    OnlyInstance(#[from] only_instance::Error),

    #[error("Failed to build event loop:\n{0}")]
    EventLoop(#[from] EventLoopError),
}
