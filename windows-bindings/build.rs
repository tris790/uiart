fn main() {
    windows::build! {
        Windows::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW,
        Windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW,
        Windows::Win32::UI::WindowsAndMessaging::WINDOW_LONG_PTR_INDEX,
        Windows::Win32::UI::KeyboardAndMouseInput::GetActiveWindow,
        Windows::Win32::UI::WindowsAndMessaging::ShowWindow,
        Windows::Win32::Graphics::Dwm::DWM_BLURBEHIND,
        Windows::Win32::Graphics::Dwm::DwmEnableBlurBehindWindow,
        Windows::Win32::Graphics::Gdi::CreateRectRgn
    };
}
