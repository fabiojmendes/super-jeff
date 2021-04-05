use glam::Vec2;

pub fn collides(pos1: Vec2, rect1: (f32, f32), pos2: Vec2, rect2: (f32, f32)) -> bool {
    (pos1.x - pos2.x).abs() < (rect1.0 + rect2.0) / 2.0
        && (pos1.y - pos2.y).abs() < (rect1.1 + rect2.1) / 2.0
}
