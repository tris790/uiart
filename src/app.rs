use std::{fs::File, time::Duration};

use sdl2::{
    event::Event,
    keyboard::Keycode,
    mouse::MouseButton,
    pixels::Color,
    rect::{Point as SdlPoint, Rect},
    render::Canvas,
    sys::{
        SDL_CreateShapedWindow, SDL_GameControllerName, SDL_GameControllerOpen, SDL_Init,
        SDL_IsGameController, SDL_NumJoysticks, SDL_bool, SDL_INIT_EVERYTHING,
    },
    video::Window,
    EventPump,
};
use windows_bindings::{
    Windows::Win32::Foundation::BOOL,
    Windows::Win32::Graphics::Dwm::DwmEnableBlurBehindWindow,
    Windows::Win32::Graphics::Dwm::DWM_BLURBEHIND,
    Windows::Win32::UI::KeyboardAndMouseInput::GetActiveWindow,
    Windows::Win32::UI::WindowsAndMessaging::SetLayeredWindowAttributes,
    Windows::Win32::UI::WindowsAndMessaging::SetWindowPos,
    Windows::Win32::UI::WindowsAndMessaging::LAYERED_WINDOW_ATTRIBUTES_FLAGS,
    Windows::Win32::UI::WindowsAndMessaging::{GetWindowLongPtrW, SetWindowLongPtrW},
    Windows::Win32::{Foundation::HWND, UI::WindowsAndMessaging::SET_WINDOW_POS_FLAGS},
    Windows::Win32::{
        Graphics::Gdi::CreateRectRgn, UI::WindowsAndMessaging::WINDOW_LONG_PTR_INDEX,
    },
};

use crate::{
    navigation::{Movement, Position},
    ui_bounding_box::UiBoundingBox,
};

pub struct State {
    id: u8,
    elements: Vec<UiBoundingBox>,
    selected: Position,
}

impl Default for State {
    fn default() -> Self {
        State {
            id: 0,
            elements: vec![],
            selected: Position { x: 0.0, y: 0.0 },
        }
    }
}

pub struct App {
    current_state: u8,
    states: Vec<State>,
    ui_dirty: bool,
    current_position: Position,
    background_color: Color,
    bounding_boxes: Vec<UiBoundingBox>,
    selected_bounding_box: Option<UiBoundingBox>,
}

const movement_length: f32 = 20.0;
const scope_angle_degree: f64 = 160.0;
const half_scope_angle_degree: f64 = scope_angle_degree / 2.0;

fn windows_specific_opacity() {
    let hwnd = unsafe { GetActiveWindow() };

    let style_index = WINDOW_LONG_PTR_INDEX::from(-16);
    let style = unsafe { GetWindowLongPtrW(hwnd, style_index) };
    let overlapped_window: isize = 0x00C00000 | 0x00080000 | 0x00040000 | 0x00020000 | 0x00010000;
    let popup: isize = 0x80000000;

    let composited: isize = 0x02000000;
    let transparent: isize = 0x00000020;

    let new_style = (style & !overlapped_window) | popup;
    unsafe { SetWindowLongPtrW(hwnd, style_index, new_style) };

    let extended_style_index = WINDOW_LONG_PTR_INDEX::from(-20);
    let extended_style = unsafe { GetWindowLongPtrW(hwnd, extended_style_index) };

    let new_extended_style = extended_style | composited | transparent;
    unsafe { SetWindowLongPtrW(hwnd, extended_style_index, new_extended_style) };

    let bb = DWM_BLURBEHIND {
        dwFlags: 1 | 2,
        fEnable: BOOL::from(true),
        hRgnBlur: unsafe { CreateRectRgn(0, 0, -1, -1) },
        fTransitionOnMaximized: BOOL::from(false),
    };
    let _ = unsafe { DwmEnableBlurBehindWindow(hwnd, &bb) };
}

fn enable_transparency() {
    let hwnd = unsafe { GetActiveWindow() };
    let GWL_EXSTYLE = WINDOW_LONG_PTR_INDEX::from(-20);
    let ex_style = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) };
    let WS_EX_LAYERED = 0x00080000;
    let WS_EX_TRANSPARENT = 0x00000020;
    let SWP_NOMOVE = 0x0002;
    let SWP_NOSIZE = 0x0001;

    unsafe {
        SetWindowLongPtrW(
            hwnd,
            GWL_EXSTYLE,
            ex_style | WS_EX_LAYERED | WS_EX_TRANSPARENT,
        )
    };
    unsafe {
        SetWindowPos(
            hwnd,
            HWND(-1),
            0,
            0,
            0,
            0,
            SET_WINDOW_POS_FLAGS::from(SWP_NOMOVE | SWP_NOSIZE),
        )
    };
    let chroma_key = 0x0000FFFF;
    unsafe {
        SetLayeredWindowAttributes(
            hwnd,
            chroma_key,
            0,
            LAYERED_WINDOW_ATTRIBUTES_FLAGS::from(1),
        )
    };
}

impl App {
    pub fn new(states: Vec<State>) -> Self {
        unsafe { SDL_Init(SDL_INIT_EVERYTHING) };
        let file = File::open("src/vscode_ui.json").unwrap();

        let mut bounding_boxes: Vec<UiBoundingBox> = serde_json::from_reader(file).unwrap();
        let mut current_position: Position = Position::new(0.0, 0.0);

        let background_color = Color::RGBA(255, 255, 0, 255);
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

        App {
            current_state: states.first().unwrap_or(&State::default()).id,
            states,
            ui_dirty,
            current_position,
            background_color,
            bounding_boxes,
            selected_bounding_box: None,
        }
    }

    pub fn set_state(&mut self, new_state: u8) {
        self.current_state = new_state;
    }

    pub fn run(&mut self) {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("uiart", 1920, 1080)
            .position_centered()
            .borderless()
            .resizable()
            .opengl()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();
        let mut event_pump = sdl_context.event_pump().unwrap();
        let mut last_angle: Option<f64> = None;
        let mut last_position: Option<Position> = None;
        enable_transparency();

        'running: loop {
            let mut movement: Option<Movement> = None;

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
                        let try_position = Position {
                            x: x as f32,
                            y: y as f32,
                        };
                        movement = Movement::maybe_new(
                            self.current_position,
                            try_position,
                            self.bounding_boxes.clone(),
                            scope_angle_degree,
                        )
                    }

                    Event::KeyDown { keycode, .. } => match keycode {
                        Some(Keycode::R) => {}
                        Some(Keycode::Right) => {
                            let try_position = Position {
                                x: self.current_position.x + movement_length,
                                y: self.current_position.y,
                            };
                            movement = Movement::maybe_new(
                                self.current_position,
                                try_position,
                                self.bounding_boxes.clone(),
                                scope_angle_degree,
                            )
                        }
                        Some(Keycode::Left) => {
                            let try_position = Position {
                                x: self.current_position.x - movement_length,
                                y: self.current_position.y,
                            };
                            movement = Movement::maybe_new(
                                self.current_position,
                                try_position,
                                self.bounding_boxes.clone(),
                                scope_angle_degree,
                            )
                        }
                        Some(Keycode::Down) => {
                            let try_position = Position {
                                x: self.current_position.x,
                                y: self.current_position.y + movement_length,
                            };
                            movement = Movement::maybe_new(
                                self.current_position,
                                try_position,
                                self.bounding_boxes.clone(),
                                scope_angle_degree,
                            )
                        }
                        Some(Keycode::Up) => {
                            let try_position = Position {
                                x: self.current_position.x,
                                y: self.current_position.y - movement_length,
                            };
                            movement = Movement::maybe_new(
                                self.current_position,
                                try_position,
                                self.bounding_boxes.clone(),
                                scope_angle_degree,
                            )
                        }
                        _ => (),
                    },
                    _ => {}
                }
            }

            canvas.set_draw_color(self.background_color);
            canvas.clear();

            // All UI elements
            canvas.set_draw_color(Color::GRAY);
            for bounding_box in &self.bounding_boxes {
                let _ = canvas.draw_rect(Rect::new(
                    bounding_box.x as i32,
                    bounding_box.y as i32,
                    bounding_box.w as u32,
                    bounding_box.h as u32,
                ));
            }

            // Current UI element
            if let Some(mov) = &movement {
                last_angle = Some(mov.angle);
                last_position = Some(mov.current_position);

                let old_position = self.current_position;
                self.current_position = mov.new_position;
                self.selected_bounding_box = mov.select_bounding_box;

                canvas.set_draw_color(Color::YELLOW);
                let _ = canvas.draw_line(
                    SdlPoint::new(old_position.x as i32, old_position.y as i32),
                    SdlPoint::new(mov.new_position.x as i32, mov.new_position.y as i32),
                );
            }

            if let Some(position) = last_position {
                if let Some(angle) = last_angle {
                    canvas.set_draw_color(Color::BLUE);
                    let angle1_position = Position {
                        x: position.x - (2000.0 * (angle - half_scope_angle_degree).cos() as f32),
                        y: position.y - (2000.0 * (angle - half_scope_angle_degree).sin() as f32),
                    };
                    let angle2_position = Position {
                        x: position.x - (2000.0 * (angle + half_scope_angle_degree).cos() as f32),
                        y: position.y - (2000.0 * (angle + half_scope_angle_degree).sin() as f32),
                    };
                    let _ = canvas.draw_line(
                        SdlPoint::new(position.x as i32, position.y as i32),
                        SdlPoint::new(angle1_position.x as i32, angle1_position.y as i32),
                    );
                    let _ = canvas.draw_line(
                        SdlPoint::new(position.x as i32, position.y as i32),
                        SdlPoint::new(angle2_position.x as i32, angle2_position.y as i32),
                    );
                }
            }

            canvas.set_draw_color(Color::GREEN);
            if let Some(selected_bounding_box) = self.selected_bounding_box {
                let _ = canvas.draw_rect(Rect::new(
                    selected_bounding_box.x as i32,
                    selected_bounding_box.y as i32,
                    selected_bounding_box.w as u32,
                    selected_bounding_box.h as u32,
                ));
            }

            canvas.present();
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
            self.ui_dirty = false;
        }
    }
}
