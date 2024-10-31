use std::{fmt::Display, time::Instant};

use uuid::Uuid;

/// Details about a break.
pub struct Break {
    /// The id of the break.
    pub id: Uuid,

    /// When the break started.
    pub started: Instant,

    /// If the break has finished, when it was finished.
    pub finished: Option<Instant>,
}

impl Break {
    /// Constructs a new break that starts now.
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            started: Instant::now(),
            finished: None,
        }
    }
}

impl Default for Break {
    fn default() -> Self {
        Self {
            id: Uuid::nil(),
            started: Instant::now(),
            finished: Some(Instant::now()),
        }
    }
}

impl Display for Break {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let finished = match self.finished.as_ref() {
            Some(finished) => format!("{} minutes ago", finished.elapsed().as_secs() / 60),
            None => "No".to_string(),
        };
        write!(
            f,
            "Break {{ id: {}, started: {} minutes ago, finished: {} }}",
            self.id,
            self.started.elapsed().as_secs() / 60,
            finished
        )
    }
}
