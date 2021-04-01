use std::time::Instant;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::WindowCanvas;

use glam::Vec2;

const WORLD_WIDTH: f32 = 32.0;
const WORLD_HEIGTH: f32 = 24.0;

const GRAVITY: f32 = -10.0;

const PLAYER_SPEED: f32 = 5.0;
const JUMP_SPEED: f32 = 10.0;

#[derive(Debug)]
struct Player {
    position: Vec2,
    side: Vec2,
    velocity: Vec2,
}

impl Player {
    fn grounded(&self) -> bool {
        self.velocity.y == 0.0
    }

    fn input(&mut self, event: &Event) {
        match event {
            Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
                self.velocity.x = -PLAYER_SPEED;
            }
            Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                self.velocity.x = PLAYER_SPEED;
            }
            Event::KeyUp { keycode: Some(Keycode::Right), .. }
            | Event::KeyUp { keycode: Some(Keycode::Left), .. } => {
                self.velocity.x = 0.0;
            }
            Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                if self.grounded() {
                    self.velocity.y = JUMP_SPEED;
                }
            }
            _ => {}
        }
    }

    fn update(&mut self, elapsed: f32) {
        // Gravity
        if !self.grounded() {
            self.velocity.y += GRAVITY * elapsed;
        }

        self.position += self.velocity * elapsed;

        // Hit the ground
        if self.position.y < 0.0 {
            self.position.y = 0.0;
            self.velocity.y = 0.0;
        }
    }
}

fn to_pixels(point: Vec2, screen_size: (u32, u32)) -> (i32, i32) {
    let w = screen_size.0 as f32;
    let h = screen_size.1 as f32;
    let t = (w / 2.0 + w * point.x / WORLD_WIDTH, h / 2.0 - h * point.y / WORLD_HEIGTH);
    (t.0 as i32, t.1 as i32)
}

fn render(canvas: &mut WindowCanvas, square: &Player) -> Result<(), String> {
    canvas.set_draw_color(Color::GRAY);
    canvas.clear();

    let size = canvas.output_size()?;
    let scale = size.0 as f32 / WORLD_WIDTH;

    let p = Point::from(to_pixels(square.position, size));
    canvas.set_draw_color(Color::RED);
    let v = square.side * scale;
    canvas.fill_rect(Rect::from_center(p, v.x as u32, v.y as u32))?;

    canvas.present();

    Ok(())
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Super Jeff", 1024, 768)
        .position_centered()
        .build()
        .expect("could not build video subsystem");

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .expect("could not build canvas from window, quiting");

    let mut timer = Instant::now();

    let mut player = Player {
        position: Vec2::new(0.0, 0.0), //
        side: Vec2::new(1.0, 2.0),     //
        velocity: Vec2::new(0.0, 0.0), //
    };

    let mut event_pump = sdl_context.event_pump()?;
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                }
                _ => {}
            }
            player.input(&event);
        }

        let delta = timer.elapsed();
        timer += delta;
        let elapsed = delta.as_millis() as f32 / 1000.0;

        player.update(elapsed);

        render(&mut canvas, &player)?;
    }

    Ok(())
}
