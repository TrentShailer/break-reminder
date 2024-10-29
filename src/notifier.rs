use std::{
    sync::mpsc::Receiver,
    thread,
    time::{Duration, Instant},
};

use tracing::{error, info};
use uuid::Uuid;
use windows::Win32::UI::WindowsAndMessaging::MB_ICONERROR;
use winit::event_loop::EventLoopProxy;

use crate::message_box::message_box;

/// Starts the notifier thread
pub fn start_notifier(proxy: EventLoopProxy<Uuid>, break_end_receiver: Receiver<Uuid>) {
    let interval_str = std::env::args().take(1).next().unwrap_or("20".to_string());
    let interval_minutes: u64 = interval_str.parse().unwrap_or(20);
    let interval = Duration::from_secs(interval_minutes * 60);

    info!("Interval set to {interval_minutes} minutes.",);

    thread::spawn(move || loop {
        thread::sleep(interval);

        let id = Uuid::new_v4();

        // Send event to window
        if let Err(e) = proxy.send_event(id) {
            error!("Failed to send event to window:\n{e}");
            message_box("Failed to send event to window", MB_ICONERROR);
            panic!("Failed to send event to window:\n{e}");
        }

        // Loop through messages until we get a response for this id
        'recv_loop: loop {
            match break_end_receiver.recv() {
                Ok(recv_id) => {
                    if recv_id == id {
                        break 'recv_loop;
                    }
                }
                Err(e) => {
                    error!("Failed to receive break env from app:\n{e}");
                    message_box("Failed to receive break env from app", MB_ICONERROR);
                    panic!("Failed to receive break env from app:\n{e}");
                }
            }
        }
    });
}
