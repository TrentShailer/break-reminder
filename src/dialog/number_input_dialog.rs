use std::cell::Cell;

use windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM, WPARAM},
    UI::WindowsAndMessaging::{
        EndDialog, GetDlgItemInt, BS_PUSHBUTTON, DLGITEMTEMPLATE, DLGTEMPLATE, DS_CENTER,
        DS_MODALFRAME, DS_SETFONT, ES_LEFT, ES_NUMBER, WM_CLOSE, WM_COMMAND, WM_INITDIALOG,
        WS_BORDER, WS_CAPTION, WS_CHILD, WS_EX_NOPARENTNOTIFY, WS_GROUP, WS_POPUPWINDOW,
        WS_TABSTOP, WS_VISIBLE,
    },
};

use crate::dialog::{
    DialogItemTemplate, DialogTemplate, CANCEL_ID, CANCEL_ID_ISIZE, INPUT_ID, OK_ID, OK_ID_ISIZE,
};

// Following are some constants that are the utf16 null-terminated strings used in the pause dialog.

const CANCEL_TITLE: [u16; 7] = [0x0043, 0x0061, 0x006e, 0x0063, 0x0065, 0x006c, 0x0000];
const CANCEL_TITLE_LENGTH: usize = CANCEL_TITLE.len();

const FONT: [u16; 9] = [
    0x0053, 0x0065, 0x0067, 0x006f, 0x0065, 0x0020, 0x0055, 0x0049, 0x0000,
];
const FONT_LENGTH: usize = FONT.len();

/// The number input dialog template is a dialog template followed by the items that make it up.
#[repr(C, align(4))]
pub struct NumberInputDialogTemplate<const T: usize, const I: usize, const S: usize> {
    pub dialog: DialogTemplate<T, FONT_LENGTH>,
    pub input_label: DialogItemTemplate<I>,
    pub input: DialogItemTemplate<1>,
    pub cancel: DialogItemTemplate<CANCEL_TITLE_LENGTH>,
    pub confirm: DialogItemTemplate<S>,
}

impl<const T: usize, const I: usize, const S: usize> NumberInputDialogTemplate<T, I, S> {
    /// # Safety
    /// - `title`, `input_title`, and `submit_title` **must** be valid null-terminated utf-16.
    pub unsafe fn new(title: [u16; T], input_title: [u16; I], submit_title: [u16; S]) -> Self {
        let dialog = DialogTemplate {
            dialog_template: DLGTEMPLATE {
                style: WS_VISIBLE.0
                    | WS_POPUPWINDOW.0
                    | WS_CAPTION.0
                    | DS_CENTER as u32
                    | DS_MODALFRAME as u32
                    | DS_SETFONT as u32,
                dwExtendedStyle: 0,
                cdit: 4,
                x: 0,
                y: 0,
                cx: 128,
                cy: 40,
            },
            menu: 0x0000,
            class: 0x0000,
            title,
            font_size: 12,
            font: FONT,
        };

        let input_label = DialogItemTemplate {
            template: DLGITEMTEMPLATE {
                style: WS_CHILD.0 | WS_VISIBLE.0 | WS_GROUP.0,
                dwExtendedStyle: WS_EX_NOPARENTNOTIFY.0,
                x: 10,
                y: 2,
                cx: 112,
                cy: 8,
                id: 0,
            },
            class: [0xFFFF, 0x0082],
            title: input_title,
            creation_data: 0x00,
        };

        let input = DialogItemTemplate {
            template: DLGITEMTEMPLATE {
                style: WS_CHILD.0
                    | WS_BORDER.0
                    | WS_VISIBLE.0
                    | WS_TABSTOP.0
                    | WS_GROUP.0
                    | ES_NUMBER as u32
                    | ES_LEFT as u32,
                dwExtendedStyle: WS_EX_NOPARENTNOTIFY.0,
                x: 8,
                y: 11,
                cx: 112,
                cy: 10,
                id: INPUT_ID,
            },
            class: [0xFFFF, 0x0081],
            title: [0x0000],
            creation_data: 0x00,
        };

        let confirm = DialogItemTemplate {
            template: DLGITEMTEMPLATE {
                style: WS_CHILD.0 | WS_VISIBLE.0 | WS_TABSTOP.0 | WS_GROUP.0 | BS_PUSHBUTTON as u32,
                dwExtendedStyle: WS_EX_NOPARENTNOTIFY.0,
                x: 8,
                y: 24,
                cx: 56,
                cy: 12,
                id: OK_ID,
            },
            class: [0xFFFF, 0x0080],
            title: submit_title,
            creation_data: 0x00,
        };

        let cancel = DialogItemTemplate {
            template: DLGITEMTEMPLATE {
                style: WS_CHILD.0 | WS_VISIBLE.0 | WS_TABSTOP.0 | WS_GROUP.0 | BS_PUSHBUTTON as u32,
                dwExtendedStyle: WS_EX_NOPARENTNOTIFY.0,
                x: 56 + 8,
                y: 24,
                cx: 56,
                cy: 12,
                id: CANCEL_ID,
            },
            class: [0xFFFF, 0x0080],
            title: CANCEL_TITLE,
            creation_data: 0x00,
        };

        NumberInputDialogTemplate {
            dialog,
            input_label,
            input,
            confirm,
            cancel,
        }
    }
}

/// Callback used by the number input, process events from the dialog.
pub extern "system" fn number_input_dialog_callback(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> isize {
    // The lparam is a *mut u32 pointer to a u32 on the main thread. This allows data to be
    // transferred from the dialog to the main thread. However, the lparam is only valid for the
    // WM_INITDIALOG message. Therefore, a thread local static is used to capture the value during
    // the message.
    thread_local! {
        pub static OUTPUT_PTR: Cell<Option<*mut u32>> = const {Cell::new(None)};
    }

    unsafe {
        match message {
            WM_INITDIALOG => {
                // Here we get the pointer and store it in the static.
                let wait_minutes_ptr = lparam.0 as *mut u32;
                OUTPUT_PTR.set(Some(wait_minutes_ptr));
            }

            WM_COMMAND => {
                let message_id = wparam.0 as isize;

                match message_id {
                    // If the message is an OK, then the value from the input is retrieved and sent
                    // to the main thread using the pointer if it exists.
                    1 | OK_ID_ISIZE => {
                        let mut result = BOOL(0);
                        let output = GetDlgItemInt(hwnd, INPUT_ID.into(), Some(&mut result), true);

                        // The result variable is false when the dialog is not a valid int.
                        // If the output is 0, we want to ignore it.
                        if !result.as_bool() || output == 0 {
                            return 0;
                        }

                        // Try get the pointer if it is set and then write to it.
                        if let Some(output_ptr) = OUTPUT_PTR.get() {
                            std::ptr::write(output_ptr, output);
                        }

                        // Close the dialog
                        EndDialog(hwnd, OK_ID_ISIZE).unwrap()
                    }

                    CANCEL_ID_ISIZE => EndDialog(hwnd, CANCEL_ID_ISIZE).unwrap(),

                    _ => {}
                }
            }

            WM_CLOSE => EndDialog(hwnd, CANCEL_ID_ISIZE).unwrap(),

            _ => {}
        }

        0
    }
}
