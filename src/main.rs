use std::collections::HashSet;
use std::time::Instant;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::WindowCanvas;

use glam::{const_vec2, Vec2};

const WORLD_WIDTH: f32 = 32.0;
const WORLD_HEIGTH: f32 = 24.0;

// Physics Constants
const GRAVITY: Vec2 = const_vec2!([0.0, -10.0]);

const PLAYER_SPEED: f32 = 25.0;
const JUMP_SPEED: f32 = 12.0;
const DRAG: f32 = 10.0;

const MAX_X_VELOCITY: f32 = 10.0;
const MAX_Y_VELOCITY: f32 = 100.0;

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

    fn speed(&self) -> f32 {
        if self.grounded() {
            PLAYER_SPEED
        } else {
            PLAYER_SPEED / 2.5
        }
    }

    fn accelerate(&mut self, vel: Vec2, elapsed: f32) {
        self.velocity += vel  * elapsed;
        if self.velocity.x.abs() > MAX_X_VELOCITY {
            self.velocity.x = MAX_X_VELOCITY.copysign(self.velocity.x);
        }
        if self.velocity.y.abs() > MAX_Y_VELOCITY {
            self.velocity.y = MAX_Y_VELOCITY.copysign(self.velocity.y);
        }
    }

    fn drag(&self) -> Vec2 {
        let drag = if self.grounded() { DRAG } else { DRAG / 4.0 };
        Vec2::new(-drag * self.velocity.x, 0.0)
    }

    fn input(&mut self, keys: HashSet<Keycode>, elapsed: f32) {
        for key in keys {
            match key {
                Keycode::Left => {
                    self.accelerate(Vec2::new(-self.speed(), 0.0), elapsed);
                }
                Keycode::Right => {
                    self.accelerate(Vec2::new(self.speed(), 0.0), elapsed);
                }
                Keycode::Space => {
                    if self.grounded() {
                        self.velocity.y = JUMP_SPEED;
                    }
                }
                _ => {}
            }
        }
    }

    fn update(&mut self, elapsed: f32) {
        println!("Grounded: {}", self.grounded());
        // Gravity
        self.accelerate(GRAVITY, elapsed);
        // Drag
        self.accelerate(self.drag(), elapsed);
        if self.velocity.x.abs() < 0.01 {
            self.velocity.x = 0.0;
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
        }
        let delta = timer.elapsed();
        timer += delta;
        let elapsed = delta.as_millis() as f32 / 1000.0;

        // Create a set of pressed Keys.
        let keys = event_pump
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();

        player.input(keys, elapsed);

        player.update(elapsed);

        render(&mut canvas, &player)?;
    }

    Ok(())
}
