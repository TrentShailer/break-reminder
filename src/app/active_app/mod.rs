use std::ffi::c_void;

use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use thiserror::Error;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    TrayIcon, TrayIconBuilder,
};
use windows::{
    core::{h, HSTRING},
    Win32::{
        Foundation::HWND,
        UI::WindowsAndMessaging::{MessageBoxW, MB_ICONINFORMATION, MB_TOPMOST},
    },
};
use winit::{
    dpi::PhysicalSize,
    error::OsError,
    event_loop::ActiveEventLoop,
    platform::windows::IconExtWindows,
    window::{BadIcon, Icon, Window},
};

/// An active initialized app that depends on the event loop.
pub struct ActiveApp {
    pub window: Window,

    #[allow(unused)]
    pub tray_icon: TrayIcon,
}

impl ActiveApp {
    pub fn new(event_loop: &ActiveEventLoop) -> Result<Self, Error> {
        let window_icon = Icon::from_resource(1, Some(PhysicalSize::new(64, 64)))?;
        let window_attributes = Window::default_attributes()
            .with_title("Break Reminder")
            .with_window_icon(Some(window_icon))
            .with_active(false)
            .with_visible(false);
        let window = event_loop.create_window(window_attributes)?;

        let tray_icon = tray_icon::Icon::from_resource(1, Some((24, 24)))?;
        let quit_item = MenuItem::with_id(1, "Quit Break Reminder", true, None);
        let tray_menu = Menu::with_items(&[&quit_item])?;
        let tooltip = format!("Break Reminder v{}", env!("CARGO_PKG_VERSION"));
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip(tooltip)
            .with_icon(tray_icon)
            .build()?;

        let app = Self { tray_icon, window };

        app.move_window_to_best_monitor();

        Ok(app)
    }

    /// moves the window the best monitor, prioritises non-primary monitors.
    pub fn move_window_to_best_monitor(&self) {
        let Some(primary_monitor) = self.window.primary_monitor() else {
            return;
        };

        let maybe_non_primary = self
            .window
            .available_monitors()
            .find(|handle| handle != &primary_monitor);

        let target_monitor = maybe_non_primary.unwrap_or(primary_monitor);
        self.window.set_outer_position(target_monitor.position());
    }

    /// Shows the break reminder
    pub fn show_break_reminder(&self) {
        let Ok(window_handle) = self.window.window_handle() else {
            return;
        };
        let raw_winow_handle = window_handle.as_raw();
        let win32_handle = match raw_winow_handle {
            RawWindowHandle::Win32(win32_window_handle) => win32_window_handle,
            _ => {
                return;
            }
        };
        let hwnd_isize = win32_handle.hwnd.get();

        let message = HSTRING::from("Take a break, stand up, drink some water, stretch.");
        unsafe {
            let a = hwnd_isize as *mut c_void;
            MessageBoxW(
                Some(&HWND(a)),
                &message,
                h!("Break Reminder"),
                MB_ICONINFORMATION | MB_TOPMOST,
            );
        }
    }

    pub fn handle_tray_icon(&self, event_loop: &ActiveEventLoop) {
        let Ok(event) = MenuEvent::receiver().try_recv() else {
            return;
        };

        #[allow(clippy::single_match)]
        match event.id.0.as_str() {
            "1" => event_loop.exit(),
            _ => {}
        }
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("Invalid Window Icon:\n{0}")]
    BadWindowIcon(#[from] BadIcon),

    #[error("Failed to create window:\n{0}")]
    CreateWindow(#[from] OsError),

    // Tray icon
    #[error("Invalid Tray Icon:\n{0}")]
    BadTrayIcon(#[from] tray_icon::BadIcon),

    #[error("Failed to build tray icon:\n{0}")]
    BuildTrayIcon(#[from] tray_icon::Error),

    #[error("Failed to create menu:\n{0}")]
    Menu(#[from] tray_icon::menu::Error),
}
