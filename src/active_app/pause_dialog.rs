use std::cell::Cell;

use windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM, WPARAM},
    UI::WindowsAndMessaging::{
        EndDialog, GetDlgItemInt, BS_PUSHBUTTON, DLGITEMTEMPLATE, DLGTEMPLATE, DS_CENTER,
        DS_MODALFRAME, DS_SETFONT, ES_LEFT, ES_NUMBER, IDCANCEL, IDH_OK, WM_CLOSE, WM_COMMAND,
        WM_INITDIALOG, WS_BORDER, WS_CAPTION, WS_CHILD, WS_EX_NOPARENTNOTIFY, WS_GROUP,
        WS_POPUPWINDOW, WS_TABSTOP, WS_VISIBLE,
    },
};

// Following are some constants that are the utf16 null-terminated strings used in the pause dialog.

const DIALOG_TITLE: [u16; 22] = [
    0x0050, 0x0061, 0x0075, 0x0073, 0x0065, 0x0020, 0x0042, 0x0072, 0x0065, 0x0061, 0x006b, 0x0020,
    0x0052, 0x0065, 0x006d, 0x0069, 0x006e, 0x0064, 0x0065, 0x0072, 0x0073, 0x0000,
];
const DIALOG_TITLE_LENGTH: usize = DIALOG_TITLE.len();

const INPUT_TITLE: [u16; 30] = [
    0x0050, 0x0061, 0x0075, 0x0073, 0x0065, 0x0020, 0x0072, 0x0065, 0x006d, 0x0069, 0x006e, 0x0064,
    0x0065, 0x0072, 0x0073, 0x0020, 0x0066, 0x006f, 0x0072, 0x0020, 0x0028, 0x006d, 0x0069, 0x006e,
    0x0075, 0x0074, 0x0065, 0x0073, 0x0029, 0x0000,
];
const INPUT_TITLE_LENGTH: usize = INPUT_TITLE.len();

const OK_TITLE: [u16; 14] = [
    0x0043, 0x006f, 0x006e, 0x0066, 0x0069, 0x0072, 0x006d, 0x0020, 0x0050, 0x0061, 0x0075, 0x0073,
    0x0065, 0x0000,
];
const OK_TITLE_LENGTH: usize = OK_TITLE.len();

const CANCEL_TITLE: [u16; 7] = [0x0043, 0x0061, 0x006e, 0x0063, 0x0065, 0x006c, 0x0000];
const CANCEL_TITLE_LENGTH: usize = CANCEL_TITLE.len();

const FONT: [u16; 9] = [
    0x0053, 0x0065, 0x0067, 0x006f, 0x0065, 0x0020, 0x0055, 0x0049, 0x0000,
];
const FONT_LENGTH: usize = FONT.len();

// IDS:

pub const CANCEL_ID: u16 = IDCANCEL.0 as u16;
pub const CANCEL_ID_ISIZE: isize = IDCANCEL.0 as isize;

pub const OK_ID: u16 = IDH_OK as u16;
pub const OK_ID_ISIZE: isize = IDH_OK as isize;

pub const INPUT_ID: u16 = 125;

/// A win32 dialog is defined in a single continous block of memory.
/// It starts structure called a [`DLGTEMPLATE`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-dlgtemplate).
/// The windows crate contains a definition for this structure as in the `dialog_template` field,
/// however, creating a dialog requires more than just the `DLGTEMPLATE`.
///
/// The function to create a dialog requres the `DLGTEMPLATE` structure to be followed immediately in
/// memory by a variable length array made up of 16-bit elements that are aligned in memory on WORD
/// boundaries. If the first element in this array is `0x0000`, then the dialog has no menu and the
/// array has no other elements. I don't expect to need a menu so this is a u16 that should always
/// be `0x0000`.
///
/// Then the menu should be followed by another variable length array, the same as above, but this
/// defines the class for the dialog. If the first element is `0x0000` then the default dialog class
/// is used. I only need the default dialog class so this is a u16 that should always be `0x0000`.
///
/// This is followed by another variable length array, the same as above, but this defines the title
/// for the dialog. This should be a utf16 encoded null-terminated string. However, this must be
/// directly after the class in memory, so a pointer to a string on the heap is not acceptable.
/// Therefore an array with a generic const length is used. If the first item in the array is `0x0000`
/// then the dialog has no title.
/// For a simplified look as to why this is complicated, a String variable is essentially a stack allocated
/// pointer, length, and capacity variables where the actual contents of the string are on the heap.
/// This is a problem because when the windows API is trying to create the dialog it is just reading
/// the bytes of memory that follow the previous bytes which if a string were used are just the pointer,
/// length, and capacity. Therefore, the title must be the contents of the title on the stack with no
/// other information. The way I acheive this is to use an array, however, array lengths must be known
/// at compile time, and therefore need to be defined as a generic parameter to the structure.
///
/// Then ONLY IF the `DLGTEMPLATE` has the `DS_SETFONT` flag in it's style, the title should be followed
/// by a 16-bit int for the size of the font which is in turn followed by a variable length array
/// the same as the title for the name of the font. Because I expect to always use the `DS_SETFONT`
/// flag, these are present in this structure.
///
/// Then based on the `cdit` field of the `DLGTEMPLATE`, the will expect the font to be followed
/// by `cdit` number of `DialogItemTemplates` in memory for the fields that make up the dialog.
#[repr(C, align(2))]
pub struct DialogTemplate<const TITLE_LENGTH: usize, const FONT_LENGTH: usize> {
    pub dialog_template: DLGTEMPLATE,

    /// Variable length array, 16-bit elements, aligned on word boundaries,
    /// If the first element of this array is 0x0000, the dialog box has no menu and the array has no other elements.
    pub menu: u16,

    /// Variable length array, 16-bit elements, aligned on word boundaries.
    /// If the first element of the array is 0x0000, the system uses the predefined dialog box class for the dialog box and the array has no other elements.
    pub class: u16,

    /// Variable length array, 16-bit elements, aligned on word boundaries.
    /// If the first element of this array is 0x0000, the dialog box has no title and the array has no other elements.
    pub title: [u16; TITLE_LENGTH],

    /// Font size
    pub font_size: u16,

    /// Font
    pub font: [u16; FONT_LENGTH],
}

/// A `DialogItemTemplate` is similar to a `DialogTemplate` as it is made up of a `DLGITEMTEMPLATE`
/// that is then followed by some dynamic length arrays.
#[repr(C, align(4))]
pub struct DialogItemTemplate<const TITLE_LENGTH: usize> {
    pub template: DLGITEMTEMPLATE,
    pub class: [u16; 2],
    pub title: [u16; TITLE_LENGTH],
    pub creation_data: u16,
}

/// The pause dialog template is a dialog template followed by the items that make it up.
#[repr(C, align(4))]
pub struct PauseDialogTemplate {
    pub dialog: DialogTemplate<DIALOG_TITLE_LENGTH, FONT_LENGTH>,
    pub input_label: DialogItemTemplate<INPUT_TITLE_LENGTH>,
    pub input: DialogItemTemplate<1>,
    pub cancel: DialogItemTemplate<CANCEL_TITLE_LENGTH>,
    pub confirm: DialogItemTemplate<OK_TITLE_LENGTH>,
}

impl PauseDialogTemplate {
    pub fn new() -> Self {
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
            title: DIALOG_TITLE,
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
            title: INPUT_TITLE,
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
            title: OK_TITLE,
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

        PauseDialogTemplate {
            dialog,
            input_label,
            input,
            confirm,
            cancel,
        }
    }
}

/// Callback used by the dialog, process events from the dialog.
pub extern "system" fn pause_dialog_callback(
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
