use glam::Vec2;
use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::TextureCreator;
use sdl2::render::{Texture, WindowCanvas};
use sdl2::video::WindowContext;

use crate::level::Level;

pub struct TextureManager<'a> {
    jeff: Texture<'a>,
    monkey: Texture<'a>,
    banana: Texture<'a>,
    tiles: Texture<'a>,
    newgame: Texture<'a>,
    gameover: Texture<'a>,
    backgrounds: Vec<Texture<'a>>,
    enemies: Vec<Texture<'a>>,
}

impl<'a> TextureManager<'a> {
    pub fn load(texture_creator: &TextureCreator<WindowContext>) -> Result<TextureManager, String> {
        let jeff = texture_creator.load_texture("assets/jeff.png")?;
        let monkey = texture_creator.load_texture("assets/monkey.png")?;
        let banana = texture_creator.load_texture("assets/banana.png")?;
        let tiles = texture_creator.load_texture("assets/tiles.png")?;
        let gameover = texture_creator.load_texture("assets/gameover.png")?;
        let newgame = texture_creator.load_texture("assets/newgame.png")?;

        let backgrounds = vec![
            texture_creator.load_texture("assets/background1.png")?,
            texture_creator.load_texture("assets/background2.png")?,
        ];

        let enemies = vec![
            texture_creator.load_texture("assets/andi.png")?,
            texture_creator.load_texture("assets/leandro.png")?,
            texture_creator.load_texture("assets/paulo.png")?,
            texture_creator.load_texture("assets/vereador.png")?,
            texture_creator.load_texture("assets/newton.png")?,
            texture_creator.load_texture("assets/be-pimp.png")?,
            texture_creator.load_texture("assets/gold.png")?,
            texture_creator.load_texture("assets/lopes.png")?,
            texture_creator.load_texture("assets/ronald.png")?,
        ];

        Ok(TextureManager { jeff, monkey, banana, tiles, newgame, gameover, backgrounds, enemies })
    }
}

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
    tx_manager: &TextureManager,
) -> Result<(), String> {
    canvas.set_draw_color(Color::RGB(178, 220, 239));
    canvas.clear();

    // Background
    for (i, bg) in tx_manager.backgrounds.iter().enumerate() {
        let (w, h) = camera.screen_size;
        let offset = if level.trapped {
            0
        } else {
            ((level.player.position.x + level.max_bounds().x) * (5.0 + i as f32 * 5.0)) as i32
                % w as i32
        };
        let dst = Rect::new(w as i32 - offset, -20, w, h);
        canvas.copy(bg, None, dst)?;
        let dst = Rect::new(-offset, -20, w, h);
        canvas.copy(bg, None, dst)?;
    }

    for t in &level.tiles {
        let p = Point::from(camera.to_pixels(t.position));
        let rect = t.sides * camera.scale();
        let src = Rect::from(t.sprite);
        let dst = Rect::from_center(p, rect.x as u32, rect.y as u32);
        canvas.copy(&tx_manager.tiles, src, dst)?;
    }

    for (i, e) in level.enemies.iter().enumerate() {
        let p = Point::from(camera.to_pixels(e.position));
        canvas.set_draw_color(Color::BLACK);
        let rect = e.sides * camera.scale();
        if !e.dead() {
            let src = Rect::from(e.sprite);
            let dst = Rect::from_center(p, rect.x as u32, rect.y as u32);
            if let Some(tex) = tx_manager.enemies.get(i % tx_manager.enemies.len()) {
                canvas.copy_ex(tex, src, dst, 0.0, None, e.velocity.x < 0.0, false)?;
            } else {
                canvas.draw_rect(Rect::from_center(p, rect.x as u32, rect.y as u32))?;
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
        // Render score
        canvas.draw_rect(Rect::from_center(p, rect.x as u32, rect.y as u32))?;
    } else {
        let src = Rect::from(level.monkey.sprite);
        let dst = Rect::from_center(p, rect.x as u32, rect.y as u32);
        canvas.copy_ex(&tx_manager.monkey, src, dst, 0.0, None, level.monkey.right(), false)?;
    }

    // Monkey Hit Box
    // let (head_pos, head_rect) = level.monkey.head();
    // let head_point = Point::from(camera.to_pixels(head_pos));
    // canvas.set_draw_color(Color::GREEN);
    // let rect = head_rect * camera.scale();
    // canvas.fill_rect(Rect::from_center(head_point, rect.x as u32, rect.y as u32))?;

    for b in &level.monkey.bananas {
        let p = Point::from(camera.to_pixels(b.position));
        canvas.set_draw_color(Color::YELLOW);
        let rect = b.sides * camera.scale();
        let dst = Rect::from_center(p, rect.x as u32, rect.y as u32);
        canvas.copy(&tx_manager.banana, None, dst)?;
    }

    let p = Point::from(camera.to_pixels(level.player.position));
    let src = Rect::from(level.player.sprite);
    let rect = level.player.sides() * camera.scale();
    let dst = Rect::from_center(p, rect.x as u32, rect.y as u32);
    canvas.copy_ex(&tx_manager.jeff, src, dst, 0.0, None, level.player.velocity.x < 0.0, false)?;

    // let (foot_pos, foot_rect) = level.player.foot_rect();
    // let foot_point = Point::from(camera.to_pixels(foot_pos));
    // canvas.set_draw_color(Color::GREEN);
    // let rect = foot_rect * camera.scale();
    // canvas.fill_rect(Rect::from_center(foot_point, rect.x as u32, rect.y as u32))?;

    // let camera_point = Point::from(camera.to_pixels(camera.center));
    // canvas.set_draw_color(Color::RED);
    // canvas.fill_rect(Rect::from_center(camera_point, 4, 4))?;

    // Overlays
    if level.player.dead {
        canvas.copy(&tx_manager.gameover, None, None)?;
    }
    if !level.started() {
        canvas.copy(&tx_manager.newgame, None, None)?;
    }

    canvas.present();

    Ok(())
}
