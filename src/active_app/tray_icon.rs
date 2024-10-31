use std::time::Duration;

use thiserror::Error;
use tracing::{error, warn};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    TrayIcon, TrayIconBuilder,
};
use windows::Win32::{
    Foundation::{GetLastError, LPARAM},
    UI::WindowsAndMessaging::DialogBoxIndirectParamW,
};
use winit::event_loop::ActiveEventLoop;

use crate::{active_app::pause_dialog::PauseDialogTemplate, message::Message};

use super::{
    pause_dialog::{pause_dialog_callback, OK_ID_ISIZE},
    ActiveApp,
};

impl ActiveApp {
    /// Creates the tray icon.
    pub(super) fn create_tray_icon() -> Result<TrayIcon, CreateError> {
        let tray_icon = tray_icon::Icon::from_resource(1, Some((24, 24)))?;

        let pause_item = MenuItem::with_id("pause", "Pause breaks for...", true, None);
        let debug_item = MenuItem::with_id("debug", "Log debug info", true, None);
        let quit_item = MenuItem::with_id("quit", "Quit Break Reminder", true, None);

        let tray_menu = Menu::with_items(&[&pause_item, &debug_item, &quit_item])?;

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

            "quit" => {
                event_loop.exit();
                None
            }

            "debug" => Some(Message::PrintDebug),

            id => {
                warn!("Unhandled tray icon event: {id}");
                None
            }
        }
    }

    /// Handles the pause dialog.
    fn show_pause_dialog(&self) -> Option<Duration> {
        unsafe {
            let template = PauseDialogTemplate::new();
            let template_pointer = std::ptr::from_ref(&template.dialog.dialog_template);

            let mut wait_minutes = Box::new(0u32);
            let wait_minutes_ptr: *mut u32 = &mut *wait_minutes;

            let result = DialogBoxIndirectParamW(
                None,
                template_pointer,
                None,
                Some(pause_dialog_callback),
                LPARAM(wait_minutes_ptr as isize),
            );

            match result {
                // -1 is an win32 error
                -1 => {
                    let error = GetLastError().0;
                    error!("Failure response from dialog:\n{error}");
                    None
                }

                OK_ID_ISIZE => {
                    let wait_minutes = *wait_minutes.as_ref() as u64;
                    Some(Duration::from_secs(60 * wait_minutes))
                }

                _ => None,
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
