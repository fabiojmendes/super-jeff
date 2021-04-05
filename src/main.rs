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

mod physics;

const CAMERA_WIDTH: f32 = 32.0;
const CAMERA_HEIGTH: f32 = 24.0;

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
    grounded: bool,
}

impl Player {
    fn grounded(&self) -> bool {
        self.grounded
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

    fn die(&mut self, spawn: Vec2) {
        self.velocity = Vec2::ZERO;
        self.position = spawn;
    }

    fn foot_rect(&self) -> (Vec2, Vec2) {
        let foot = Vec2::new(self.position.x, self.position.y - self.side.y / 2.0 - 0.08);
        (foot, Vec2::new(0.55, 0.05))
    }

    fn update(&mut self, keys: &HashSet<Keycode>, elapsed: f32, level: &mut Level) {
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

        self.grounded = false;
        // Check for collisions
        for t in &level.tiles {
            let x_collision = physics::collides(
                self.position + Vec2::new(displacement.x, 0.0),
                self.side.into(),
                t.position,
                (t.side, t.side),
            );

            if x_collision {
                displacement.x = 0.0;
                self.velocity.x = 0.0;
            }

            let y_collision = physics::collides(
                self.position + Vec2::new(0.0, displacement.y),
                self.side.into(),
                t.position,
                (t.side, t.side),
            );

            if y_collision {
                displacement.y = 0.0;
                self.velocity.y = 0.0;
            }

            // Foot Collision
            let (foot_pos, foot_rect) = self.foot_rect();
            if physics::collides(foot_pos, foot_rect.into(), t.position, (t.side, t.side)) {
                self.grounded = true;
            }
        }
        // Apply new Position
        self.position += displacement;

        let (foot_pos, foot_rect) = self.foot_rect();
        level
            .enemies
            .retain(|e| !physics::collides(foot_pos, foot_rect.into(), e.position, e.side.into()));

        for e in &level.enemies {
            if physics::collides(self.position, self.side.into(), e.position, e.side.into()) {
                self.die(level.spawn);
            }
        }

        // Reset if it falls
        if self.position.y < -(level.height as f32) {
            self.die(level.spawn);
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

#[derive(Debug)]
struct Camera {
    center: Vec2,
    bounds: Vec2,
    screen_size: (u32, u32),
}

impl Camera {
    fn to_pixels(&self, point: Vec2) -> (i32, i32) {
        let w = self.screen_size.0 as f32;
        let h = self.screen_size.1 as f32;
        let point = point - self.center;
        let t = (w / 2.0 + w * point.x / self.bounds.x, h / 2.0 - h * point.y / self.bounds.y);
        (t.0 as i32, t.1 as i32)
    }

    fn recenter(&mut self, position: Vec2, level_bounds: (f32, f32)) {
        self.center = position;
        // Clamp camera position
        let level_bounds = Vec2::from(level_bounds);
        let max = level_bounds / 2.0 - self.bounds / 2.0;
        self.center = self.center.clamp(-max, max);
    }

    fn scale(&self) -> f32 {
        self.screen_size.0 as f32 / self.bounds.x
    }
}

fn render(
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
        canvas.fill_rect(Rect::from_center(
            pos,
            (t.side * camera.scale()) as u32,
            (t.side * camera.scale()) as u32,
        ))?;
    }

    for e in &level.enemies {
        let p = Point::from(camera.to_pixels(e.position));
        canvas.set_draw_color(Color::BLACK);
        let rect = e.side * camera.scale();
        canvas.fill_rect(Rect::from_center(p, rect.x as u32, rect.y as u32))?;
    }

    let p = Point::from(camera.to_pixels(player.position));
    canvas.set_draw_color(Color::BLUE);
    let rect = player.side * camera.scale();
    canvas.fill_rect(Rect::from_center(p, rect.x as u32, rect.y as u32))?;

    let (foot_pos, foot_rect) = player.foot_rect();
    let foot_point = Point::from(camera.to_pixels(foot_pos));
    canvas.set_draw_color(Color::GREEN);
    let rect = foot_rect * camera.scale();
    canvas.fill_rect(Rect::from_center(foot_point, rect.x as u32, rect.y as u32))?;

    let camera_point = Point::from(camera.to_pixels(camera.center));
    canvas.set_draw_color(Color::RED);
    canvas.fill_rect(Rect::from_center(camera_point, 16, 16))?;

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

    let mut level = Level::from_file("level.txt") //
        .expect("Error loading level from file");

    let mut player = Player {
        position: level.spawn,
        side: Vec2::new(0.9, 1.8),
        velocity: Vec2::new(0.0, 0.0),
        grounded: false,
    };

    let mut camera = Camera {
        center: player.position,
        bounds: Vec2::new(CAMERA_WIDTH, CAMERA_HEIGTH),
        screen_size: canvas.output_size()?,
    };

    println!("Player {:?}, Camera {:?}", player, camera);

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

        player.update(&keys, elapsed, &mut level);

        level.update(elapsed);

        camera.recenter(player.position, (level.width as f32, level.height as f32));

        render(&mut canvas, &camera, &player, &level)?;
    }

    Ok(())
}
