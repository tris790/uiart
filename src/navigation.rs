use std::cmp::Ordering;

use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};

use crate::{point::Point, ui_bounding_box::UiBoundingBox};

pub struct Movement {
    pub p1: Point,
    pub p2: Point,
}

pub struct Position {
    pub value: Point,
}

impl Position {
    pub fn move_to(&mut self, new_position: Position) {
        self.value = new_position.value;
    }
}

pub struct PointWithDistance {
    pub p: Point,
    pub bounding_box: UiBoundingBox,
    pub distance: f64,
}

pub fn find_center_of_bounding_box(bounding_box: &UiBoundingBox) -> Point {
    Point {
        x: bounding_box.w / 2.0 + bounding_box.x,
        y: bounding_box.h / 2.0 + bounding_box.y,
    }
}

pub fn distance_two_points(p1: &Point, p2: &Point) -> f64 {
    (((p2.x - p1.x).powi(2) + (p2.y - p1.y)) as f64).sqrt()
}

pub fn distance_between_line_point(movement: &Movement, point: &Point) -> f64 {
    let sum_p1 = distance_two_points(&movement.p1, point);
    let sum_p2 = distance_two_points(&movement.p2, point);
    return sum_p1 + sum_p2;
}

pub fn filter_by_angle(
    movement: &Movement,
    bounding_boxes: Vec<UiBoundingBox>,
    angle: f64,
) -> Vec<UiBoundingBox> {
    let x2: f64 = (movement.p2.x - movement.p1.x) as f64;
    let y2: f64 = (movement.p2.y - movement.p1.y) as f64;
    let target_angle: f64 = y2.atan2(x2);
    println!(
        "---------target angle {}-------------",
        target_angle.to_degrees()
    );

    let total_angle_scope: f64 = angle.to_radians();
    let half_angle_scope: f64 = total_angle_scope / 2.0;
    let min_scope_angle: f64 = target_angle - half_angle_scope;
    let max_scope_angle: f64 = target_angle + half_angle_scope;

    bounding_boxes
        .into_iter()
        .filter(|bounding_box| {
            let box_center = find_center_of_bounding_box(&bounding_box);

            if movement.p1.x == box_center.x && movement.p1.y == box_center.y {
                return false;
            }

            let x: f64 = (box_center.x - movement.p1.x) as f64;
            let y: f64 = (box_center.y - movement.p1.y) as f64;
            println!(
                "----box [{} {}] mov [{} {}] me [{} {}]",
                box_center.x,
                box_center.y,
                movement.p2.x,
                movement.p2.y,
                movement.p1.x,
                movement.p1.y
            );
            let bounding_box_angle = y.atan2(x);

            println!(
                "    my_angle {} bounds [{} {}]",
                bounding_box_angle.to_degrees(),
                min_scope_angle.to_degrees(),
                max_scope_angle.to_degrees()
            );
            println!(
                "can goto {}",
                bounding_box_angle >= min_scope_angle && bounding_box_angle <= max_scope_angle
            );
            return bounding_box_angle >= min_scope_angle && bounding_box_angle <= max_scope_angle;
        })
        .collect()
}

pub fn find_nearest_ui_element(
    movement: &Movement,
    elements: &Vec<UiBoundingBox>,
) -> Option<PointWithDistance> {
    elements
        .iter()
        .map(|bounding_box| {
            let p: Point = find_center_of_bounding_box(&bounding_box);
            let distance: f64 = distance_between_line_point(&movement, &p);
            PointWithDistance {
                p,
                distance,
                bounding_box: bounding_box.clone(),
            }
        })
        .min_by(|x: &PointWithDistance, y: &PointWithDistance| {
            match x.distance.partial_cmp(&y.distance) {
                Some(ord) => ord,
                None => Ordering::Greater,
            }
        })
}
