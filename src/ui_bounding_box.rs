use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct UiBoundingBox {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}
