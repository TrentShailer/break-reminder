use std::{
    fmt::Display,
    time::{Duration, Instant},
};

/// Details about an *ongoing* or *previous* pause.
pub struct Pause {
    /// How long the pause should last for.
    pub duration: Duration,

    /// When the pause was started.
    pub started: Instant,
}

impl Pause {
    /// If the pause is ongoing or not.
    pub fn is_active(&self) -> bool {
        self.started.elapsed() < self.duration
    }
}

impl Display for Pause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Pause {{ duration: {} minutes, started: {} minutes ago, is_active: {} }}",
            self.duration.as_secs() / 60,
            self.started.elapsed().as_secs() / 60,
            self.is_active()
        )
    }
}
