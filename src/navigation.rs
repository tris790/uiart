use std::cmp::Ordering;

use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};

use crate::ui_bounding_box::UiBoundingBox;

pub struct Movement {
    pub current_position: Position,
    pub new_position: Position,
    pub select_bounding_box: Option<UiBoundingBox>,
    pub angle: f64,
}

impl Movement {
    pub fn maybe_new(
        current_position: Position,
        new_position: Position,
        bounding_boxes: Vec<UiBoundingBox>,
        scope_angle: f64,
    ) -> Option<Movement> {
        let x2: f64 = (new_position.x - current_position.x) as f64;
        let y2: f64 = (new_position.y - current_position.y) as f64;
        let angle: f64 = y2.atan2(x2);
        let movement = Movement {
            current_position,
            new_position,
            select_bounding_box: None,
            angle,
        };

        let filtered_bounding_box = movement.filter_by_angle(bounding_boxes, scope_angle);
        if filtered_bounding_box.len() > 0 {
            println!("Count: {}", filtered_bounding_box.len());
            let nearest_element_result = movement.find_nearest_ui_element(&filtered_bounding_box);
            let movement_output = match nearest_element_result {
                Some(nearest_element_result) => Some(Movement {
                    current_position,
                    new_position: nearest_element_result.p,
                    select_bounding_box: Some(nearest_element_result.bounding_box),
                    angle,
                }),
                None => None,
            };
            return movement_output;
        }
        None
    }

    fn filter_by_angle(
        &self,
        bounding_boxes: Vec<UiBoundingBox>,
        scope_angle: f64,
    ) -> Vec<UiBoundingBox> {
        println!(
            "---------target angle {}-------------",
            self.angle.to_degrees()
        );

        let total_angle_scope: f64 = scope_angle.to_radians();
        let half_angle_scope: f64 = total_angle_scope / 2.0;
        let mut min_scope_angle: f64 = self.angle - half_angle_scope;
        let mut max_scope_angle: f64 = self.angle + half_angle_scope;
        // if min_scope_angle < 0.0 {
        //     min_scope_angle += 360.0;
        // }

        // if max_scope_angle < 0.0 {
        //     max_scope_angle += 360.0;
        // }

        bounding_boxes
            .into_iter()
            .filter(|bounding_box| {
                let box_center = find_center_of_bounding_box(&bounding_box);

                if self.current_position.x == box_center.x
                    && self.current_position.y == box_center.y
                {
                    return false;
                }

                let x: f64 = (box_center.x - self.current_position.x) as f64;
                let y: f64 = (box_center.y - self.current_position.y) as f64;
                println!(
                    "----box({}) [{} {}] mov [{} {}] me [{} {}]",
                    bounding_box.id,
                    box_center.x,
                    box_center.y,
                    self.new_position.x,
                    self.new_position.y,
                    self.current_position.x,
                    self.current_position.y
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
                return bounding_box_angle >= min_scope_angle
                    && bounding_box_angle <= max_scope_angle;
            })
            .collect()
    }

    fn find_nearest_ui_element(
        &self,
        elements: &Vec<UiBoundingBox>,
    ) -> Option<NearestElementResult> {
        elements
            .iter()
            .map(|bounding_box| {
                let test_bounding_box_position: Position =
                    find_center_of_bounding_box(&bounding_box);
                let distance: f64 =
                    distance_two_positions(&self.new_position, &test_bounding_box_position);
                println!(
                    "[{}, {}] point: {} is {} away from point [{},{}]",
                    self.new_position.x,
                    self.new_position.y,
                    bounding_box.id,
                    distance,
                    test_bounding_box_position.x,
                    test_bounding_box_position.y
                );
                NearestElementResult {
                    p: test_bounding_box_position,
                    distance,
                    bounding_box: bounding_box.clone(),
                }
            })
            .min_by(|x: &NearestElementResult, y: &NearestElementResult| {
                match x.distance.partial_cmp(&y.distance) {
                    Some(ord) => ord,
                    None => Ordering::Greater,
                }
            })
    }
}

#[derive(Clone, Copy)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Position { x, y }
    }
}

pub struct NearestElementResult {
    pub p: Position,
    pub bounding_box: UiBoundingBox,
    pub distance: f64,
}

pub fn find_center_of_bounding_box(bounding_box: &UiBoundingBox) -> Position {
    Position {
        x: bounding_box.w / 2.0 + bounding_box.x,
        y: bounding_box.h / 2.0 + bounding_box.y,
    }
}

pub fn distance_two_positions(p1: &Position, p2: &Position) -> f64 {
    ((p2.x - p1.x).powi(2) + (p2.y - p1.y).powi(2)).sqrt() as f64
}
