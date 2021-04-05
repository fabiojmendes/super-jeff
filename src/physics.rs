use glam::{const_vec2, Vec2};

pub const GRAVITY: Vec2 = const_vec2!([0.0, -25.0]);

pub const DRAG: f32 = 3.0;

pub fn collides(pos1: Vec2, rect1: (f32, f32), pos2: Vec2, rect2: (f32, f32)) -> bool {
    (pos1.x - pos2.x).abs() < (rect1.0 + rect2.0) / 2.0
        && (pos1.y - pos2.y).abs() < (rect1.1 + rect2.1) / 2.0
}
