fn main() {
    windows::build! {
        Windows::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW,
        Windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW,
        Windows::Win32::UI::WindowsAndMessaging::WINDOW_LONG_PTR_INDEX,
        Windows::Win32::UI::KeyboardAndMouseInput::GetActiveWindow,
        Windows::Win32::UI::WindowsAndMessaging::ShowWindow,
        Windows::Win32::Graphics::Dwm::DWM_BLURBEHIND,
        Windows::Win32::Graphics::Dwm::DwmEnableBlurBehindWindow,
        Windows::Win32::Graphics::Gdi::CreateRectRgn,
        Windows::Win32::UI::WindowsAndMessaging::SetLayeredWindowAttributes,
        Windows::Win32::UI::WindowsAndMessaging::LAYERED_WINDOW_ATTRIBUTES_FLAGS,
        Windows::Win32::UI::WindowsAndMessaging::SetWindowPos,
        Windows::Win32::UI::WindowsAndMessaging::SET_WINDOW_POS_FLAGS
    };
}
