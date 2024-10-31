use std::ffi::c_void;

use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use thiserror::Error;
use windows::Win32::Foundation::HWND;
use winit::{
    dpi::PhysicalSize,
    error::OsError,
    event_loop::ActiveEventLoop,
    platform::windows::IconExtWindows,
    window::{BadIcon, Icon, Window},
};

use super::ActiveApp;

impl ActiveApp {
    /// Creates the application window.
    pub(super) fn create_window(event_loop: &ActiveEventLoop) -> Result<Window, CreateError> {
        let window_icon = Icon::from_resource(1, Some(PhysicalSize::new(64, 64)))?;

        let window_attributes = Window::default_attributes()
            .with_title("Break Reminder")
            .with_window_icon(Some(window_icon))
            .with_active(false)
            .with_visible(false);

        let window = event_loop.create_window(window_attributes)?;

        Ok(window)
    }

    /// Tries to get the HWND from the window.
    pub unsafe fn get_hwnd(&self) -> Option<HWND> {
        let window_handle = self.window.window_handle().ok()?;
        let raw_winow_handle = window_handle.as_raw();

        let win32_handle = match raw_winow_handle {
            RawWindowHandle::Win32(win32_window_handle) => win32_window_handle,
            _ => return None,
        };

        let hwnd_isize = win32_handle.hwnd.get();

        Some(HWND(hwnd_isize as *mut c_void))
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
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CreateError {
    #[error("Invalid Icon:\n{0}")]
    BadIcon(#[from] BadIcon),

    #[error("Failed to create window:\n{0}")]
    CreateWindow(#[from] OsError),
}
