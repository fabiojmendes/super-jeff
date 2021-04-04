use std::collections::HashSet;
use std::time::Instant;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::WindowCanvas;

use glam::{const_vec2, Vec2};

mod level;
use level::Level;

const WORLD_WIDTH: f32 = 32.0;
const WORLD_HEIGTH: f32 = 24.0;

// Physics Constants
const GRAVITY: Vec2 = const_vec2!([0.0, -25.0]);

const PLAYER_SPEED: f32 = 30.0;
const JUMP_SPEED: f32 = 15.0;
const DRAG: f32 = 3.0;

const MAX_VELOCITY: Vec2 = const_vec2!([10.0, 100.0]);

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
            PLAYER_SPEED / 4.0
        }
    }

    fn accelerate(&mut self, vel: Vec2, elapsed: f32) {
        self.velocity += vel * elapsed;
        self.velocity = self.velocity.clamp(-MAX_VELOCITY, MAX_VELOCITY);
    }

    fn apply_drag(&mut self, elapsed: f32) {
        let drag = if self.grounded() { DRAG } else { 0.0 };
        self.accelerate(Vec2::new(-drag * self.velocity.x, 0.0), elapsed);
        if self.velocity.x.abs() < 0.1 {
            self.velocity.x = 0.0;
        }
    }

    fn die(&mut self) {
        self.velocity = Vec2::new(0.0, 0.0);
        self.position = Vec2::new(0.0, 0.0);
    }

    fn update(&mut self, keys: &HashSet<Keycode>, elapsed: f32, level: &Level) {
        // Drag
        self.apply_drag(elapsed);

        // Input
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

        // Gravity
        self.accelerate(GRAVITY, elapsed);

        let mut displacement = self.velocity * elapsed;

        // Check for collisions
        for t in &level.tiles {
            let x_collision = collides(
                self.position + Vec2::new(displacement.x, 0.0),
                self.side.into(),
                t.position,
                (t.side, t.side),
            );

            if x_collision {
                displacement.x = 0.0;
                self.velocity.x = 0.0;
            }

            let y_collision = collides(
                self.position + Vec2::new(0.0, displacement.y),
                self.side.into(),
                t.position,
                (t.side, t.side),
            );

            if y_collision {
                displacement.y = 0.0;
                self.velocity.y = 0.0;
            }
        }
        // Apply new Position
        self.position += displacement;

        for e in &level.enemies {
            if collides(self.position, self.side.into(), e.position, e.side.into()) {
                self.die();
            }
        }

        // Reset if it falls
        if self.position.y < -(WORLD_HEIGTH) {
            self.die();
        }
    }

    // fn state(&self) -> PlayerState {
    //     use PlayerState::*;
    //     match self.velocity {
    //         v if v.y.abs() > 0.1 => Jumping,
    //         v if v.x > 0.1 => WalkingRight,
    //         v if v.x < -0.1 => WalkingLeft,
    //         _ => Standing,
    //     }
    // }
}

// #[derive(Debug)]
// enum PlayerState {
//     WalkingLeft,
//     WalkingRight,
//     Jumping,
//     Standing,
// }

fn collides(pos1: Vec2, rect1: (f32, f32), pos2: Vec2, rect2: (f32, f32)) -> bool {
    (pos1.x - pos2.x).abs() < (rect1.0 + rect2.0) / 2.0
        && (pos1.y - pos2.y).abs() < (rect1.1 + rect2.1) / 2.0
}

fn to_pixels(point: Vec2, camera: Vec2, screen_size: (u32, u32)) -> (i32, i32) {
    let w = screen_size.0 as f32;
    let h = screen_size.1 as f32;
    let point = point - camera;
    let t = (w / 2.0 + w * point.x / WORLD_WIDTH, h / 2.0 - h * point.y / WORLD_HEIGTH);
    (t.0 as i32, t.1 as i32)
}

fn render(
    canvas: &mut WindowCanvas,
    camera: Vec2,
    player: &Player,
    level: &Level,
) -> Result<(), String> {
    canvas.set_draw_color(Color::GRAY);
    canvas.clear();

    let size = canvas.output_size()?;
    let scale = size.0 as f32 / WORLD_WIDTH;

    for t in &level.tiles {
        canvas.set_draw_color(Color::RGB(127, 0, 0));
        let pos = Point::from(to_pixels(t.position, camera, size));
        canvas.fill_rect(Rect::from_center(
            pos,
            (t.side * scale) as u32,
            (t.side * scale) as u32,
        ))?;
    }

    for e in &level.enemies {
        let p = Point::from(to_pixels(e.position, camera, size));
        canvas.set_draw_color(Color::BLACK);
        let rect = e.side * scale;
        canvas.fill_rect(Rect::from_center(p, rect.x as u32, rect.y as u32))?;
    }

    let p = Point::from(to_pixels(player.position, camera, size));
    canvas.set_draw_color(Color::BLUE);
    let rect = player.side * scale;
    canvas.fill_rect(Rect::from_center(p, rect.x as u32, rect.y as u32))?;

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
        position: Vec2::new(0.0, 0.0),
        side: Vec2::new(0.9, 1.8),
        velocity: Vec2::new(0.0, 0.0),
    };

    let mut level = Level::from_file("level.txt", (WORLD_WIDTH, WORLD_HEIGTH))
        .expect("Error loading level from file");

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
        let elapsed = delta.as_millis() as f32 / 1000.0;
        timer += delta;

        // Create a set of pressed Keys.
        let keys = event_pump
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();

        player.update(&keys, elapsed, &level);

        level.update(elapsed);

        let mut camera = player.position.clone();
        if camera.x < 0.0 {
            camera.x = 0.0;
        }
        if camera.x > level.width as f32 - WORLD_WIDTH {
            camera.x = level.width as f32 - WORLD_WIDTH;
        }
        if camera.y < 0.0 {
            camera.y = 0.0;
        }
        if camera.y > level.height as f32 - WORLD_HEIGTH {
            camera.y = level.height as f32 - WORLD_HEIGTH;
        }

        render(&mut canvas, camera, &player, &level)?;
    }

    Ok(())
}
