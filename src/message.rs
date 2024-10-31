use std::time::Duration;

use uuid::Uuid;

/// Message variants between notifier and event loop.
#[derive(Debug)]
#[non_exhaustive]
pub enum Message {
    /// Message to let the notifier know the break has ended.
    EndBreak(Uuid),

    /// Message to make the notifier pause sending notificaitons for a given time.
    PauseReminders(Duration),

    /// Message to print the current status of the notifier.
    PrintDebug,
}
