use std::time::Duration;

use tracing::error;
use windows::Win32::{
    Foundation::{GetLastError, LPARAM},
    UI::WindowsAndMessaging::DialogBoxIndirectParamW,
};

use crate::dialog::{
    number_input_dialog::{number_input_dialog_callback, NumberInputDialogTemplate},
    OK_ID_ISIZE,
};

use super::ActiveApp;

const DIALOG_TITLE: [u16; 24] = [
    0x0042, 0x0072, 0x0065, 0x0061, 0x006b, 0x0020, 0x0052, 0x0065, 0x006d, 0x0069, 0x006e, 0x0064,
    0x0065, 0x0072, 0x0020, 0x0049, 0x006e, 0x0074, 0x0065, 0x0072, 0x0076, 0x0061, 0x006c, 0x0000,
];

const INPUT_TITLE: [u16; 23] = [
    0x0053, 0x0065, 0x0074, 0x0020, 0x0069, 0x006e, 0x0074, 0x0065, 0x0072, 0x0076, 0x0061, 0x006c,
    0x0020, 0x0028, 0x006d, 0x0069, 0x006e, 0x0075, 0x0074, 0x0065, 0x0073, 0x0029, 0x0000,
];

const SUBMIT_TITLE: [u16; 17] = [
    0x0043, 0x006f, 0x006e, 0x0066, 0x0069, 0x0072, 0x006d, 0x0020, 0x0049, 0x006e, 0x0074, 0x0065,
    0x0072, 0x0076, 0x0061, 0x006c, 0x0000,
];

impl ActiveApp {
    /// Handles the interval dialog.
    pub(super) fn show_interval_dialog(&self) -> Option<Duration> {
        unsafe {
            let template = NumberInputDialogTemplate::new(DIALOG_TITLE, INPUT_TITLE, SUBMIT_TITLE);
            let template_pointer = std::ptr::from_ref(&template.dialog.dialog_template);

            let mut wait_minutes = Box::new(0u32);
            let wait_minutes_ptr: *mut u32 = &mut *wait_minutes;

            let result = DialogBoxIndirectParamW(
                None,
                template_pointer,
                None,
                Some(number_input_dialog_callback),
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
