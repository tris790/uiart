use std::cmp::Ordering;

use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct UiBoundingBox {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

pub struct Movement {
    pub p1: Point,
    pub p2: Point,
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
    let target_angle = y2.atan2(x2);

    let total_angle_scope: f64 = 45f64.to_radians();
    let half_angle_scope: f64 = total_angle_scope / 2.0;
    let min_scope_angle: f64 = target_angle - half_angle_scope;
    let max_scope_angle: f64 = target_angle + half_angle_scope;

    bounding_boxes
        .into_iter()
        .filter(|bounding_box| {
            let x: f64 = (bounding_box.x - movement.p1.x) as f64;
            let y: f64 = (bounding_box.y - movement.p1.y) as f64;
            let bounding_box_angle = y.atan2(x);

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
