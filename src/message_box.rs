use windows::{
    core::{h, HSTRING},
    Win32::UI::WindowsAndMessaging::{MessageBoxW, MESSAGEBOX_STYLE},
};

pub fn message_box<S: Into<String>>(message: S, style: MESSAGEBOX_STYLE) {
    let message = HSTRING::from(message.into());
    unsafe {
        MessageBoxW(None, &message, h!("Break Reminder"), style);
    }
}
