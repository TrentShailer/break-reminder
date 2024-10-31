mod pause_dialog;
mod tray_icon;
mod window;

use thiserror::Error;

use ::tray_icon::TrayIcon;
use windows::{
    core::{h, HSTRING},
    Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONINFORMATION, MB_TOPMOST},
};
use winit::{event_loop::ActiveEventLoop, window::Window};

/// An active initialized app that depends on the event loop.
pub struct ActiveApp {
    pub window: Window,

    #[allow(unused)]
    pub tray_icon: TrayIcon,
}

impl ActiveApp {
    pub fn new(event_loop: &ActiveEventLoop) -> Result<Self, Error> {
        let window = Self::create_window(event_loop)?;
        let tray_icon = Self::create_tray_icon()?;

        let app = Self { tray_icon, window };

        app.move_window_to_best_monitor();

        Ok(app)
    }

    /// Shows the break reminder
    pub fn show_break_reminder(&self) {
        let maybe_hwnd = unsafe { self.get_hwnd() };
        let Some(hwnd) = maybe_hwnd else { return };

        let message = HSTRING::from("Take a break, stand up, drink some water, stretch.");
        unsafe {
            MessageBoxW(
                Some(&hwnd),
                &message,
                h!("Break Reminder"),
                MB_ICONINFORMATION | MB_TOPMOST,
            );
        }
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("Failed to create the window:\n{0}")]
    CreateWindow(#[from] window::CreateError),

    #[error("Failed to create tray icon:\n{0}")]
    CreateTrayIcon(#[from] tray_icon::CreateError),
}
