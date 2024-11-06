use thiserror::Error;
use tracing::{error, warn};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    TrayIcon, TrayIconBuilder,
};
use winit::event_loop::ActiveEventLoop;

use crate::message::Message;

use super::ActiveApp;

impl ActiveApp {
    /// Creates the tray icon.
    pub(super) fn create_tray_icon() -> Result<TrayIcon, CreateError> {
        let tray_icon = tray_icon::Icon::from_resource(1, Some((24, 24)))?;

        let pause_item = MenuItem::with_id("pause", "Pause breaks for...", true, None);
        let interval_item = MenuItem::with_id("interval", "Set interval...", true, None);
        let debug_log_item = MenuItem::with_id("debug_log", "Log debug info", true, None);
        let debug_show_item = MenuItem::with_id("debug_show", "Show debug info", true, None);
        let quit_item = MenuItem::with_id("quit", "Quit Break Reminder", true, None);

        let tray_menu = Menu::with_items(&[
            &pause_item,
            &interval_item,
            &debug_show_item,
            &debug_log_item,
            &quit_item,
        ])?;

        let tooltip = format!("Break Reminder v{}", env!("CARGO_PKG_VERSION"));

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip(tooltip)
            .with_icon(tray_icon)
            .build()?;

        Ok(tray_icon)
    }

    /// Handles any tray icon events.
    pub fn handle_tray_icon(&self, event_loop: &ActiveEventLoop) -> Option<Message> {
        let Ok(event) = MenuEvent::receiver().try_recv() else {
            return None;
        };

        match event.id.0.as_str() {
            "pause" => self.show_pause_dialog().map(Message::PauseReminders),

            "interval" => self.show_interval_dialog().map(Message::SetInterval),

            "quit" => {
                event_loop.exit();
                None
            }

            "debug_log" => Some(Message::PrintDebug),

            "debug_show" => Some(Message::ShowDebug),

            id => {
                warn!("Unhandled tray icon event: {id}");
                None
            }
        }
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CreateError {
    #[error("Invalid Icon:\n{0}")]
    BadIcon(#[from] tray_icon::BadIcon),

    #[error("Failed to build tray icon:\n{0}")]
    BuildTrayIcon(#[from] tray_icon::Error),

    #[error("Failed to create menu:\n{0}")]
    CreateMenu(#[from] tray_icon::menu::Error),
}
