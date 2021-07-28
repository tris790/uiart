use std::{cmp::Ordering, error::Error, fs::File, io::Read, process::exit};

use app::App;
use windows_bindings::{
    Windows::Win32::Foundation::BOOL,
    Windows::Win32::Graphics::Dwm::DwmEnableBlurBehindWindow,
    Windows::Win32::Graphics::Dwm::DWM_BLURBEHIND,
    Windows::Win32::UI::KeyboardAndMouseInput::GetActiveWindow,
    Windows::Win32::UI::WindowsAndMessaging::{GetWindowLongPtrW, SetWindowLongPtrW},
    Windows::Win32::{
        Graphics::Gdi::CreateRectRgn, UI::WindowsAndMessaging::WINDOW_LONG_PTR_INDEX,
    },
};

extern crate sdl2;

mod app;
mod navigation;
mod point;
mod ui_bounding_box;
fn windows_specific_opacity() {
    let hwnd = unsafe { GetActiveWindow() };

    let style_index = WINDOW_LONG_PTR_INDEX::from(-16);
    let style = unsafe { GetWindowLongPtrW(hwnd, style_index) };
    let overlapped_window: isize = 0x00C00000 | 0x00080000 | 0x00040000 | 0x00020000 | 0x00010000;
    let popup: isize = 0x80000000;

    let composited: isize = 0x02000000;
    let transparent: isize = 0x00000020;

    // let new_style = (style & !overlapped_window) | popup;
    // unsafe { SetWindowLongPtrW(hwnd, style_index, new_style) };

    let extended_style_index = WINDOW_LONG_PTR_INDEX::from(-20);
    let extended_style = unsafe { GetWindowLongPtrW(hwnd, extended_style_index) };

    // let new_extended_style = extended_style | composited | transparent;
    // unsafe { SetWindowLongPtrW(hwnd, extended_style_index, new_extended_style) };

    let bb = DWM_BLURBEHIND {
        dwFlags: 1 | 2,
        fEnable: BOOL::from(true),
        hRgnBlur: unsafe { CreateRectRgn(0, 0, -1, -1) },
        fTransitionOnMaximized: BOOL::from(false),
    };
    let _ = unsafe { DwmEnableBlurBehindWindow(hwnd, &bb) };
}

pub fn main() {
    let mut app = App::new(vec![]);
    app.run();
}
