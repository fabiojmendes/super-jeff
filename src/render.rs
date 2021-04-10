use glam::Vec2;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Texture, WindowCanvas};

use crate::level::Level;

#[derive(Debug)]
pub struct Camera {
    center: Vec2,
    bounds: Vec2,
    screen_size: (u32, u32),
}

impl Camera {
    const WIDTH: f32 = 32.0;
    const HEIGTH: f32 = 18.0;

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
        let max = level_bounds - self.bounds / 2.0;
        self.center = self.center.clamp(-max, max);
    }

    pub fn scale(&self) -> f32 {
        self.screen_size.0 as f32 / self.bounds.x
    }
}

pub fn render(
    canvas: &mut WindowCanvas,
    camera: &Camera,
    level: &Level,
    textures: &Vec<Texture>,
) -> Result<(), String> {
    canvas.set_draw_color(Color::GRAY);
    canvas.clear();

    for t in &level.tiles {
        canvas.set_draw_color(Color::RGB(127, 0, 0));
        let p = Point::from(camera.to_pixels(t.position));
        let rect = t.sides * camera.scale();
        canvas.draw_rect(Rect::from_center(p, rect.x as u32, rect.y as u32))?;
        canvas.draw_point(p)?;
    }
    let enemy_textures = &textures[3..];

    for (i, e) in level.enemies.iter().enumerate() {
        let p = Point::from(camera.to_pixels(e.position));
        canvas.set_draw_color(Color::BLACK);
        let rect = e.sides * camera.scale();
        if !e.dead() {
            let src = Rect::from(e.sprite);
            let dst = Rect::from_center(p, rect.x as u32, rect.y as u32);
            if let Some(tx) = enemy_textures.get(i % enemy_textures.len()) {
                canvas.copy_ex(tx, src, dst, 0.0, None, e.velocity.x < 0.0, false)?;
            }
            // // Hit box
            // canvas.draw_rect(Rect::from_center(p, rect.x as u32, rect.y as u32))?;

            // let (head_pos, head_rect) = e.head();
            // let head_point = Point::from(camera.to_pixels(head_pos));
            // canvas.set_draw_color(Color::GREEN);
            // let rect = head_rect * camera.scale();
            // canvas.fill_rect(Rect::from_center(head_point, rect.x as u32, rect.y as u32))?;
        }
    }

    let p = Point::from(camera.to_pixels(level.monkey.position));
    let color = if level.monkey.enranged { Color::RED } else { Color::YELLOW };
    canvas.set_draw_color(color);
    let rect = level.monkey.sides * camera.scale();
    if level.monkey.dead() {
        canvas.draw_rect(Rect::from_center(p, rect.x as u32, rect.y as u32))?;
    } else {
        let src = Rect::from(level.monkey.sprite);
        let dst = Rect::from_center(p, rect.x as u32, rect.y as u32);
        if let Some(tx) = textures.get(1) {
            canvas.copy_ex(tx, src, dst, 0.0, None, level.monkey.right(), false)?;
        }
    }
    let (head_pos, head_rect) = level.monkey.head();
    let head_point = Point::from(camera.to_pixels(head_pos));
    canvas.set_draw_color(Color::GREEN);
    let rect = head_rect * camera.scale();
    canvas.fill_rect(Rect::from_center(head_point, rect.x as u32, rect.y as u32))?;

    for b in &level.monkey.bananas {
        let p = Point::from(camera.to_pixels(b.position));
        canvas.set_draw_color(Color::YELLOW);
        let rect = b.sides * camera.scale();
        let dst = Rect::from_center(p, rect.x as u32, rect.y as u32);
        if let Some(tx) = textures.get(2) {
            canvas.copy(tx, None, dst)?;
        }
    }

    let p = Point::from(camera.to_pixels(level.player.position));
    let src = Rect::from(level.player.sprite);
    let rect = level.player.sides() * camera.scale();
    let dst = Rect::from_center(p, rect.x as u32, rect.y as u32);
    if let Some(tx) = textures.get(0) {
        canvas.copy_ex(tx, src, dst, 0.0, None, level.player.velocity.x < 0.0, false)?;
    }

    // let (foot_pos, foot_rect) = level.player.foot_rect();
    // let foot_point = Point::from(camera.to_pixels(foot_pos));
    // canvas.set_draw_color(Color::GREEN);
    // let rect = foot_rect * camera.scale();
    // canvas.fill_rect(Rect::from_center(foot_point, rect.x as u32, rect.y as u32))?;

    // let camera_point = Point::from(camera.to_pixels(camera.center));
    // canvas.set_draw_color(Color::RED);
    // canvas.fill_rect(Rect::from_center(camera_point, 4, 4))?;

    canvas.present();

    Ok(())
}
