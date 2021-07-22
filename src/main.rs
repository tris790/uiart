use std::{cmp::Ordering, error::Error, fs::File, io::Read, process::exit};

use navigation::PointWithDistance;
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
use sdl2::{
    event::Event,
    mouse::MouseButton,
    rect::{Point as SdlPoint, Rect},
    sys::{SDL_GameControllerName, SDL_Init, SDL_NumJoysticks, SDL_bool, SDL_INIT_EVERYTHING},
};
use sdl2::{keyboard::Keycode, sys::SDL_GameControllerOpen};
use sdl2::{pixels::Color, sys::SDL_IsGameController};
use std::time::Duration;

mod navigation;
use crate::navigation::{filter_by_angle, find_nearest_ui_element, Movement, Point, UiBoundingBox};

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
    unsafe { SDL_Init(SDL_INIT_EVERYTHING) };
    let file = File::open("src/data.json").unwrap();

    let bounding_boxes: Vec<UiBoundingBox> = serde_json::from_reader(file).unwrap();
    let mut current_position: Movement = Movement {
        p1: Point { x: 0.0, y: 0.0 },
        p2: Point { x: 1.0, y: 1.0 },
    };

    let mut filtered_bounding_box: Vec<UiBoundingBox> =
        filter_by_angle(&current_position, bounding_boxes.clone(), 45.0);

    let mut selected_bounding_box =
        find_nearest_ui_element(&current_position, &filtered_bounding_box);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("UIart", 1280, 800)
        .position_centered()
        .resizable()
        .vulkan()
        .build()
        .unwrap();

    windows_specific_opacity();

    let mut canvas = window.into_canvas().build().unwrap();
    let background_color = Color::RGBA(0, 0, 0, 255);
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i = 0;
    println!("Controllers: {:?}", unsafe { SDL_NumJoysticks() });
    unsafe {
        for i in 0..SDL_NumJoysticks() {
            println!("Controller: {:?}", i);

            if SDL_IsGameController(i) == SDL_bool::SDL_TRUE {
                let controller = SDL_GameControllerOpen(i);
                let controller_name = SDL_GameControllerName(controller);
                println!("Controller: {:?}", controller_name);
            }
        }
    }

    let mut ui_dirty = true;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::JoyButtonDown { .. } => println!("down"),
                Event::JoyDeviceAdded { .. } => println!("added"),
                Event::JoyDeviceRemoved { .. } => println!("removed"),
                Event::JoyAxisMotion { .. } => println!("motion"),
                Event::JoyBallMotion { .. } => println!("ball"),
                Event::MouseButtonDown {
                    mouse_btn, x, y, ..
                } => {
                    if mouse_btn == MouseButton::Left {
                        ui_dirty = true;
                        current_position.p1.x = x as f32;
                        current_position.p1.y = y as f32;
                    }

                    if mouse_btn == MouseButton::Right {
                        ui_dirty = true;
                        current_position.p2.x = x as f32;
                        current_position.p2.y = y as f32;
                    }
                }

                Event::KeyDown { keycode, .. } => match keycode {
                    Some(Keycode::Right) => {
                        println!(
                            "Current pos [{} {}]",
                            current_position.p1.x, current_position.p1.y
                        );
                        ui_dirty = true;
                        current_position.p2.x = current_position.p1.x + 10.0;
                        println!(
                            "Mov pos [{} {}]",
                            current_position.p2.x, current_position.p2.y
                        );
                    }
                    Some(Keycode::Left) => {
                        println!(
                            "Current pos [{} {}]",
                            current_position.p1.x, current_position.p1.y
                        );
                        ui_dirty = true;
                        current_position.p2.x = current_position.p1.x - 10.0;
                        println!(
                            "Mov pos [{} {}]",
                            current_position.p2.x, current_position.p2.y
                        );
                    }
                    Some(Keycode::Down) => {
                        println!(
                            "Current pos [{} {}]",
                            current_position.p1.x, current_position.p1.y
                        );
                        ui_dirty = true;
                        current_position.p2.y = current_position.p1.y + 10.0;
                        println!(
                            "Mov pos [{} {}]",
                            current_position.p2.x, current_position.p2.y
                        );
                    }
                    Some(Keycode::Up) => {
                        println!(
                            "Current pos [{} {}]",
                            current_position.p1.x, current_position.p1.y
                        );
                        ui_dirty = true;
                        current_position.p2.y = current_position.p1.y - 10.0;
                        println!(
                            "Mov pos [{} {}]",
                            current_position.p2.x, current_position.p2.y
                        );
                    }

                    _ => (),
                },
                _ => {}
            }
        }

        if ui_dirty {
            println!("{} elements left", filtered_bounding_box.len());
            filtered_bounding_box =
                filter_by_angle(&current_position, bounding_boxes.clone(), 45.0);
            println!("{} elements left", filtered_bounding_box.len());

            match find_nearest_ui_element(&current_position, &filtered_bounding_box) {
                Some(nearest_element) => {
                    current_position.p1 = nearest_element.p;
                    current_position.p2 = nearest_element.p;
                    selected_bounding_box = Some(nearest_element);
                }
                None => (),
            };
        }

        canvas.set_draw_color(background_color);
        canvas.clear();

        // All UI elements
        canvas.set_draw_color(Color::GRAY);
        for bounding_box in &bounding_boxes {
            let _ = canvas.draw_rect(Rect::new(
                bounding_box.x as i32,
                bounding_box.y as i32,
                bounding_box.w as u32,
                bounding_box.h as u32,
            ));
        }

        // Neighbouring UI elements
        canvas.set_draw_color(Color::BLUE);
        for bounding_box in &filtered_bounding_box {
            let _ = canvas.draw_rect(Rect::new(
                bounding_box.x as i32,
                bounding_box.y as i32,
                bounding_box.w as u32,
                bounding_box.h as u32,
            ));
        }

        // Current UI element
        canvas.set_draw_color(Color::GREEN);
        if let Some(selected_element) = &selected_bounding_box {
            let _ = canvas.draw_rect(Rect::new(
                selected_element.bounding_box.x as i32,
                selected_element.bounding_box.y as i32,
                selected_element.bounding_box.w as u32,
                selected_element.bounding_box.h as u32,
            ));
        }

        // Movement
        canvas.set_draw_color(Color::YELLOW);
        let _ = canvas.draw_line(
            SdlPoint::new(current_position.p1.x as i32, current_position.p1.y as i32),
            SdlPoint::new(current_position.p2.x as i32, current_position.p2.y as i32),
        );

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        i += 1;
        ui_dirty = false;
    }
}
