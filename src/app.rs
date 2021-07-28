use std::{fs::File, time::Duration};

use sdl2::{
    event::Event,
    keyboard::Keycode,
    mouse::MouseButton,
    pixels::Color,
    rect::Point as SdlPoint,
    render::Canvas,
    sys::{
        SDL_GameControllerName, SDL_GameControllerOpen, SDL_Init, SDL_IsGameController,
        SDL_NumJoysticks, SDL_bool, SDL_INIT_EVERYTHING,
    },
    video::Window,
    EventPump,
};

use crate::{navigation::Movement, point::Point, ui_bounding_box::UiBoundingBox};

pub struct State {
    id: u8,
    elements: Vec<UiBoundingBox>,
    selected: Point,
}

impl Default for State {
    fn default() -> Self {
        State {
            id: 0,
            elements: vec![],
            selected: Point { x: 0.0, y: 0.0 },
        }
    }
}

pub struct App {
    current_state: u8,
    states: Vec<State>,
    ui_dirty: bool,
    current_position: Movement,
    background_color: Color,
}

const movement_length: f32 = 100.0;

impl App {
    pub fn new(states: Vec<State>) -> Self {
        let scope_angle_degree = 160.0;

        unsafe { SDL_Init(SDL_INIT_EVERYTHING) };
        let file = File::open("src/data.json").unwrap();

        let mut bounding_boxes: Vec<UiBoundingBox> = serde_json::from_reader(file).unwrap();
        let mut current_position: Movement = Movement {
            p1: Point { x: 0.0, y: 0.0 },
            p2: Point { x: 1.0, y: 1.0 },
        };

        let background_color = Color::RGBA(0, 0, 0, 255);
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
        }
    }

    pub fn set_state(&mut self, new_state: u8) {
        self.current_state = new_state;
    }

    fn handle_mouse_event(&mut self, mouse_btn: MouseButton, x: i32, y: i32) {
        if mouse_btn == MouseButton::Left {
            self.ui_dirty = true;
            self.current_position.p1.x = x as f32;
            self.current_position.p1.y = y as f32;
        }

        if mouse_btn == MouseButton::Right {
            self.ui_dirty = true;
            self.current_position.p2.x = x as f32;
            self.current_position.p2.y = y as f32;
        }
    }

    fn handle_move_right(&mut self) {
        println!(
            "Current pos [{} {}]",
            self.current_position.p1.x, self.current_position.p1.y
        );
        self.ui_dirty = true;
        self.current_position.p2.x = self.current_position.p1.x + movement_length;
        println!(
            "Mov pos [{} {}]",
            self.current_position.p2.x, self.current_position.p2.y
        );
    }

    fn handle_move_left(&mut self) {
        println!(
            "Current pos [{} {}]",
            self.current_position.p1.x, self.current_position.p1.y
        );
        self.ui_dirty = true;
        self.current_position.p2.x = self.current_position.p1.x - movement_length;
        println!(
            "Mov pos [{} {}]",
            self.current_position.p2.x, self.current_position.p2.y
        );
    }

    fn handle_move_up(&mut self) {
        println!(
            "Current pos [{} {}]",
            self.current_position.p1.x, self.current_position.p1.y
        );
        self.ui_dirty = true;
        self.current_position.p2.y = self.current_position.p1.y - movement_length;
        println!(
            "Mov pos [{} {}]",
            self.current_position.p2.x, self.current_position.p2.y
        );
    }

    fn handle_move_down(&mut self) {
        println!(
            "Current pos [{} {}]",
            self.current_position.p1.x, self.current_position.p1.y
        );
        self.ui_dirty = true;
        self.current_position.p2.y = self.current_position.p1.y + movement_length;
        println!(
            "Mov pos [{} {}]",
            self.current_position.p2.x, self.current_position.p2.y
        );
    }

    pub fn run(&mut self) {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("UIart", 1280, 800)
            .position_centered()
            .resizable()
            .vulkan()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();
        let mut event_pump = sdl_context.event_pump().unwrap();
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
                        self.handle_mouse_event(mouse_btn, x, y);
                    }

                    Event::KeyDown { keycode, .. } => match keycode {
                        Some(Keycode::R) => {}
                        Some(Keycode::Right) => {
                            self.handle_move_right();
                        }
                        Some(Keycode::Left) => {
                            self.handle_move_left();
                        }
                        Some(Keycode::Down) => {
                            self.handle_move_down();
                        }
                        Some(Keycode::Up) => {
                            self.handle_move_up();
                        }
                        _ => (),
                    },
                    _ => {}
                }
            }

            if self.ui_dirty {
                // match find_nearest_ui_element(&self.current_position, &filtered_bounding_box) {
                //     Some(nearest_element) => {
                //         self.current_position.p1 = nearest_element.p;
                //         self.current_position.p2 = nearest_element.p;
                //         selected_bounding_box = Some(nearest_element);
                //     }
                //     None => (),
                // };
            }

            canvas.set_draw_color(self.background_color);
            canvas.clear();

            // All UI elements
            canvas.set_draw_color(Color::GRAY);
            // for bounding_box in &bounding_boxes {
            //     let _ = canvas.draw_rect(Rect::new(
            //         bounding_box.x as i32,
            //         bounding_box.y as i32,
            //         bounding_box.w as u32,
            //         bounding_box.h as u32,
            //     ));
            // }

            // Current UI element
            canvas.set_draw_color(Color::GREEN);
            // if let Some(selected_element) = &selected_bounding_box {
            //     let _ = canvas.draw_rect(Rect::new(
            //         selected_element.bounding_box.x as i32,
            //         selected_element.bounding_box.y as i32,
            //         selected_element.bounding_box.w as u32,
            //         selected_element.bounding_box.h as u32,
            //     ));
            // }

            // Movement
            canvas.set_draw_color(Color::YELLOW);
            let _ = canvas.draw_line(
                SdlPoint::new(
                    self.current_position.p1.x as i32,
                    self.current_position.p1.y as i32,
                ),
                SdlPoint::new(
                    self.current_position.p2.x as i32,
                    self.current_position.p2.y as i32,
                ),
            );

            canvas.present();
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
            self.ui_dirty = false;
        }
    }
}
