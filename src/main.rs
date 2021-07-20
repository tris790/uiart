use std::{cmp::Ordering, error::Error, fs::File, io::Read, process::exit};

use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};

use macroquad::prelude::*;

use windows_bindings::{
    Windows::Win32::Graphics::Dwm::DwmEnableBlurBehindWindow,
    Windows::Win32::Graphics::Dwm::DWM_BLURBEHIND,
    Windows::Win32::UI::KeyboardAndMouseInput::GetActiveWindow,
    Windows::Win32::{Foundation::BOOL, UI::WindowsAndMessaging::ShowWindow},
    Windows::Win32::{
        Graphics::Gdi::CreateRectRgn,
        UI::WindowsAndMessaging::{SHOW_WINDOW_CMD, WINDOW_LONG_PTR_INDEX},
    },
    Windows::Win32::{
        Graphics::Gdi::HRGN,
        UI::WindowsAndMessaging::{GetWindowLongPtrW, SetWindowLongPtrW},
    },
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
struct UiBoundingBox {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

#[derive(Debug)]
struct Point {
    x: f32,
    y: f32,
}

#[derive(Debug)]
struct Line {
    p1: Point,
    p2: Point,
}

#[derive(Debug)]
struct PointWithDistance {
    p: Point,
    bounding_box: UiBoundingBox,
    distance: f64,
}

fn find_center_of_bounding_box(bounding_box: &UiBoundingBox) -> Point {
    Point {
        x: bounding_box.w / 2.0 + bounding_box.x,
        y: bounding_box.h / 2.0 + bounding_box.y,
    }
}

fn distance_two_points(p1: &Point, p2: &Point) -> f64 {
    (((p2.x - p1.x).powi(2) + (p2.y - p1.y)) as f64).sqrt()
}

fn distance_between_line_point(line: &Line, point: &Point) -> f64 {
    let sum_p1 = distance_two_points(&line.p1, point);
    let sum_p2 = distance_two_points(&line.p2, point);
    return sum_p1 + sum_p2;
}

fn filter_45angle(line: &Line, bounding_boxes: Vec<UiBoundingBox>) -> Vec<UiBoundingBox> {
    let x2: f64 = (line.p2.x - line.p1.x) as f64;
    let y2: f64 = (line.p2.y - line.p1.y) as f64;
    let target_angle = y2.atan2(x2);

    let total_angle_scope: f64 = 45f64.to_radians();
    let half_angle_scope: f64 = total_angle_scope / 2.0;
    let min_scope_angle: f64 = target_angle - half_angle_scope;
    let max_scope_angle: f64 = target_angle + half_angle_scope;

    bounding_boxes
        .into_iter()
        .filter(|bounding_box| {
            let x: f64 = (bounding_box.x - line.p1.x) as f64;
            let y: f64 = (bounding_box.y - line.p1.y) as f64;
            let bounding_box_angle = y.atan2(x);

            return bounding_box_angle >= min_scope_angle && bounding_box_angle <= max_scope_angle;
        })
        .collect()
}

fn find_nearest_ui_element(
    user_line: &Line,
    elements: &Vec<UiBoundingBox>,
) -> Option<PointWithDistance> {
    elements
        .iter()
        .map(|bounding_box| {
            let p: Point = find_center_of_bounding_box(&bounding_box);
            let distance: f64 = distance_between_line_point(&user_line, &p);
            PointWithDistance {
                p,
                distance,
                bounding_box: bounding_box.clone(),
            }
        })
        .min_by(|x: &PointWithDistance, y: &PointWithDistance| {
            x.distance.partial_cmp(&y.distance).unwrap()
        })
}

fn window_conf() -> Conf {
    Conf {
        window_title: "test".to_owned(),
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let file = File::open("src/data.json").unwrap();

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

    let new_extended_style = extended_style | composited | transparent;
    unsafe { SetWindowLongPtrW(hwnd, extended_style_index, new_extended_style) };

    let bb = DWM_BLURBEHIND {
        dwFlags: 1 | 2,
        fEnable: BOOL::from(true),
        hRgnBlur: unsafe { CreateRectRgn(0, 0, -1, -1) },
        fTransitionOnMaximized: BOOL::from(false),
    };
    let _ = unsafe { DwmEnableBlurBehindWindow(hwnd, &bb) };

    // let closest_point: PointWithDistance = filtered_bounding_box
    //     .iter()
    //     .map(|bounding_box| {
    //         let p: Point = find_center_of_bounding_box(&bounding_box);
    //         let distance: f64 = distance_between_line_point(&user_line, &p);
    //         PointWithDistance {
    //             p,
    //             distance,
    //             bounding_box: bounding_box.clone(),
    //         }
    //     })
    //     .min_by(|x: &PointWithDistance, y: &PointWithDistance| {
    //         x.distance.partial_cmp(&y.distance).unwrap()
    //     })
    //     .unwrap();

    let bounding_boxes: Vec<UiBoundingBox> = serde_json::from_reader(file).unwrap();
    let mut user_line: Line = Line {
        p1: Point { x: 0.0, y: 0.0 },
        p2: Point { x: 50.0, y: 0.0 },
    };

    let mut filtered_bounding_box: Vec<UiBoundingBox> =
        filter_45angle(&user_line, bounding_boxes.clone());

    let mut selected_bounding_box = match find_nearest_ui_element(&user_line, &filtered_bounding_box) {
        Some(element) => element.bounding_box,
        None => None
    }

    loop {
        clear_background(Color::from_rgba(0, 0, 0, 255));
        if is_mouse_button_down(MouseButton::Left) {
            let mouse_position = mouse_position();
            user_line.p1.x = mouse_position.0;
            user_line.p1.y = mouse_position.1;
            filtered_bounding_box = filter_45angle(&user_line, bounding_boxes.clone());
            selected_bounding_box = find_nearest_ui_element(&user_line, &filtered_bounding_box);
        }
        if is_mouse_button_down(MouseButton::Right) {
            let mouse_position = mouse_position();
            user_line.p2.x = mouse_position.0;
            user_line.p2.y = mouse_position.1;
            filtered_bounding_box = filter_45angle(&user_line, bounding_boxes.clone());
            selected_bounding_box = find_nearest_ui_element(&user_line, &filtered_bounding_box);
        }
        // println!("Closest {:?}", closest_point);

        for bounding_box in &bounding_boxes {
            draw_rectangle(
                bounding_box.x,
                bounding_box.y,
                bounding_box.w,
                bounding_box.h,
                GRAY,
            );
        }

        for bounding_box in &filtered_bounding_box {
            draw_rectangle(
                bounding_box.x,
                bounding_box.y,
                bounding_box.w,
                bounding_box.h,
                GREEN,
            );
        }

        draw_rectangle(
            selected_bounding_box.x,
            selected_bounding_box.y,
            selected_bounding_box.w,
            selected_bounding_box.h,
            BLUE,
        );

        draw_line(
            user_line.p1.x,
            user_line.p1.y,
            user_line.p2.x,
            user_line.p2.y,
            10.0,
            YELLOW,
        );

        next_frame().await
    }
}
