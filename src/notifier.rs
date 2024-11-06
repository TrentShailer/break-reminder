use std::{
    sync::mpsc::{Receiver, TryRecvError},
    thread::{self},
    time::{Duration, Instant},
};

use tracing::{info, warn};
use uuid::Uuid;
use winit::event_loop::EventLoopProxy;

use crate::{break_reminder::Break, message::Message, pause::Pause};

/// The notifier object, operates on a separate thread to main event loop.
pub struct Notifier {
    /// Proxy to the event loop.
    proxy: EventLoopProxy<Uuid>,

    /// Receiever for messages from the event loop.
    message_receiver: Receiver<Message>,

    /// The interval between notifications.
    interval: Duration,

    /// The paused details of the notifier.
    paused: Option<Pause>,

    /// The last break
    last_break: Break,
}

struct ShouldCloseThread;

impl Notifier {
    pub fn new(
        proxy: EventLoopProxy<Uuid>,
        message_receiver: Receiver<Message>,
        interval: Duration,
    ) -> Self {
        Self {
            proxy,
            message_receiver,
            interval,
            paused: None,
            last_break: Break::default(),
        }
    }

    /// Tries to load the interval from the program arguments.
    pub fn interval_from_args() -> Option<Duration> {
        let interval_str = std::env::args().last()?;
        let interval_minutes: u64 = interval_str.parse().ok()?;
        Some(Duration::from_secs(interval_minutes * 60))
    }

    /// This starts the event loop on another thread, takes ownership of the notifier.
    pub fn start_event_loop(mut self) {
        thread::spawn(move || loop {
            if self.handle_events().is_err() {
                return;
            };

            if self.should_notify() {
                let send_result = self.send_reminder();
                if send_result.is_err() {
                    return;
                };
            }

            thread::sleep(Duration::from_millis(100));
        });
    }

    /// Handle any incoming events from the message receiver.
    fn handle_events(&mut self) -> Result<(), ShouldCloseThread> {
        let message = match self.message_receiver.try_recv() {
            Ok(message) => message,
            Err(e) => match e {
                TryRecvError::Empty => return Ok(()),
                TryRecvError::Disconnected => {
                    // The sender no longer exists, we should shut down.
                    warn!("Message sender has disconnected, shutting down notifier");
                    return Err(ShouldCloseThread);
                }
            },
        };

        match message {
            Message::EndBreak(uuid) => {
                if self.last_break.id == uuid {
                    self.last_break.finished = Some(Instant::now())
                } else {
                    warn!(
                        "End break message's ID ({}) does not match the last break ID ({})",
                        uuid, self.last_break.id
                    );
                }
            }

            Message::PauseReminders(duration) => {
                let pause = Pause {
                    duration,
                    started: Instant::now(),
                };
                self.paused = Some(pause);
            }

            Message::PrintDebug => {
                info!("Interval: {} minutes", self.interval.as_secs() / 60);
                match self.paused.as_ref() {
                    Some(pause) => info!("{pause}"),
                    None => info!("Paused: No"),
                };
                info!("{}", self.last_break);
            }
        }

        Ok(())
    }

    /// Returns if the notifier should send a break notification.
    fn should_notify(&self) -> bool {
        if self.is_paused() {
            return false;
        }

        match self.last_break.finished {
            Some(finished_at) => finished_at.elapsed() >= self.interval,

            None => false,
        }
    }

    /// Returns if the notifier is paused.
    fn is_paused(&self) -> bool {
        let Some(pause) = self.paused.as_ref() else {
            return false;
        };
        pause.is_active()
    }

    /// Sends a reminder to the event loop.
    fn send_reminder(&mut self) -> Result<(), ShouldCloseThread> {
        self.last_break = Break::new();

        let send_result = self.proxy.send_event(self.last_break.id);
        if send_result.is_err() {
            warn!("Event loop has closed, notifer will shut down");
            return Err(ShouldCloseThread);
        }

        Ok(())
    }
}
