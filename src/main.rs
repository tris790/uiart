use std::{cmp::Ordering, error::Error, fs::File, io::Read, process::exit};

use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};

#[derive(Serialize, Deserialize, Debug)]
struct UiBoundingBox {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
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
        x: bounding_box.w / 2 + bounding_box.x,
        y: bounding_box.h / 2 + bounding_box.y,
    }
}

fn distance_two_points(p1: &Point, p2: &Point) -> f64 {
    (((p2.x - p1.x).pow(2) + (p2.y - p1.y)) as f64).sqrt()
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

fn main() -> Result<()> {
    let file = File::open("src/data.json").unwrap();
    let bounding_boxes: Vec<UiBoundingBox> = serde_json::from_reader(file)?;
    let user_line: Line = Line {
        p1: Point { x: 100, y: 0 },
        p2: Point { x: 110, y: 0 },
    };

    let filtered_bounding_box: Vec<UiBoundingBox> = filter_45angle(&user_line, bounding_boxes);
    println!(
        "had these elements after filtering: {:?}",
        filtered_bounding_box
    );
    if filtered_bounding_box.len() == 0 {
        println!("No element found");
        exit(0);
    }
    let closest_point: PointWithDistance = filtered_bounding_box
        .into_iter()
        .map(|bounding_box: UiBoundingBox| {
            let p: Point = find_center_of_bounding_box(&bounding_box);
            let distance: f64 = distance_between_line_point(&user_line, &p);
            PointWithDistance {
                p,
                distance,
                bounding_box,
            }
        })
        .min_by(|x: &PointWithDistance, y: &PointWithDistance| {
            x.distance.partial_cmp(&y.distance).unwrap()
        })
        .unwrap();

    println!("Closest {:?}", closest_point);
    Ok(())
}
