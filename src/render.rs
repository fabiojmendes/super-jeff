use glam::Vec2;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::WindowCanvas;

use crate::level::Level;
use crate::player::Player;

#[derive(Debug)]
pub struct Camera {
    center: Vec2,
    bounds: Vec2,
    screen_size: (u32, u32),
}

impl Camera {
    const WIDTH: f32 = 32.0;
    const HEIGTH: f32 = 24.0;

    pub fn new(size: (u32, u32)) -> Camera {
        Camera {
            center: Vec2::ZERO,
            bounds: Vec2::new(Camera::WIDTH, Camera::HEIGTH),
            screen_size: size,
        }
    }

    pub fn to_pixels(&self, point: Vec2) -> (i32, i32) {
        let w = self.screen_size.0 as f32;
        let h = self.screen_size.1 as f32;
        let point = point - self.center;
        let t = (w / 2.0 + w * point.x / self.bounds.x, h / 2.0 - h * point.y / self.bounds.y);
        (t.0 as i32, t.1 as i32)
    }

    pub fn recenter(&mut self, position: Vec2, level_bounds: Vec2) {
        self.center = position;
        // Clamp camera position
        let max = level_bounds / 2.0 - self.bounds / 2.0;
        self.center = self.center.clamp(-max, max);
    }

    pub fn scale(&self) -> f32 {
        self.screen_size.0 as f32 / self.bounds.x
    }
}

pub fn render(
    canvas: &mut WindowCanvas,
    camera: &Camera,
    player: &Player,
    level: &Level,
) -> Result<(), String> {
    canvas.set_draw_color(Color::GRAY);
    canvas.clear();

    for t in &level.tiles {
        canvas.set_draw_color(Color::RGB(127, 0, 0));
        let pos = Point::from(camera.to_pixels(t.position));
        let rect = t.sides * camera.scale();
        canvas.fill_rect(Rect::from_center(pos, rect.x as u32, rect.y as u32))?;
    }

    for e in &level.enemies {
        let p = Point::from(camera.to_pixels(e.position));
        canvas.set_draw_color(Color::BLACK);
        let rect = e.sides * camera.scale();
        canvas.fill_rect(Rect::from_center(p, rect.x as u32, rect.y as u32))?;
    }

    let p = Point::from(camera.to_pixels(level.monkey.position));
    canvas.set_draw_color(Color::YELLOW);
    let rect = level.monkey.sides * camera.scale();
    canvas.fill_rect(Rect::from_center(p, rect.x as u32, rect.y as u32))?;

    let p = Point::from(camera.to_pixels(player.position));
    canvas.set_draw_color(Color::BLUE);
    let rect = player.sides * camera.scale();
    canvas.fill_rect(Rect::from_center(p, rect.x as u32, rect.y as u32))?;

    let (foot_pos, foot_rect) = player.foot_rect();
    let foot_point = Point::from(camera.to_pixels(foot_pos));
    canvas.set_draw_color(Color::GREEN);
    let rect = foot_rect * camera.scale();
    canvas.fill_rect(Rect::from_center(foot_point, rect.x as u32, rect.y as u32))?;

    // let camera_point = Point::from(camera.to_pixels(camera.center));
    // canvas.set_draw_color(Color::RED);
    // canvas.fill_rect(Rect::from_center(camera_point, 16, 16))?;

    canvas.present();

    Ok(())
}
