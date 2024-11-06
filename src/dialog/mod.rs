pub mod number_input_dialog;

use windows::Win32::UI::WindowsAndMessaging::{DLGITEMTEMPLATE, DLGTEMPLATE, IDCANCEL, IDH_OK};

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
#[repr(C, align(4))]
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
