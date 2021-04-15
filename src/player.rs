use glam::{const_vec2, Vec2};
use sdl2::keyboard::Keycode;
use std::collections::HashSet;
use std::time::Instant;

use crate::level::Tile;
use crate::physics;
use crate::sound::SoundEffect;

const PLAYER_SPEED: f32 = 30.0;
const JUMP_SPEED: f32 = 15.0;

const MAX_VELOCITY: Vec2 = const_vec2!([10.0, 100.0]);

#[derive(Debug)]
pub struct Player {
    pub spawn: Vec2,
    pub position: Vec2,
    pub sides: Vec2,
    pub velocity: Vec2,
    pub dead: bool,
    grounded: bool,
    crouched: bool,
    pub sprite: (i32, i32, u32, u32),
    timer: Instant,
}

impl Player {
    pub fn new() -> Player {
        Player {
            spawn: Vec2::ZERO,
            position: Vec2::ZERO,
            sides: Vec2::new(0.9, 1.8),
            velocity: Vec2::ZERO,
            dead: false,
            grounded: false,
            crouched: false,
            sprite: (0, 0, 128, 256),
            timer: Instant::now(),
        }
    }

    fn grounded(&self) -> bool {
        self.grounded
    }

    pub fn sides(&self) -> Vec2 {
        if self.crouched {
            self.sides * Vec2::new(1.0, 0.5)
        } else {
            self.sides
        }
    }

    pub fn attack(&mut self, position: Vec2, sides: Vec2) -> bool {
        let (foot_pos, foot_rect) = self.foot_rect();
        let attacked =
            self.velocity.y < 0.0 && physics::collides(foot_pos, foot_rect, position, sides);
        if attacked {
            self.jump();
        }
        attacked
    }

    fn jump(&mut self) {
        self.velocity.y = JUMP_SPEED;
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
        let drag = if self.grounded() { physics::DRAG } else { 0.0 };
        self.accelerate(Vec2::new(-drag * self.velocity.x, 0.0), elapsed);
        if self.velocity.x.abs() < 0.1 {
            self.velocity.x = 0.0;
        }
    }

    pub fn die(&mut self) {
        self.dead = true;
        self.velocity = Vec2::ZERO;
    }

    pub fn foot_rect(&self) -> (Vec2, Vec2) {
        let foot = Vec2::new(self.position.x, self.position.y - self.sides.y / 2.0);
        (foot, Vec2::new(self.sides.x, 0.2))
    }

    pub fn hitbox(&self) -> Vec2 {
        self.sides - Vec2::new(0.25, 0.25)
    }

    pub fn update(
        &mut self,
        keys: &HashSet<Keycode>,
        elapsed: f32,
        tiles: &Vec<Tile>,
        sounds: &mut Vec<SoundEffect>,
    ) {
        // Drag
        self.apply_drag(elapsed);

        self.crouched = false;
        // Input
        for key in keys {
            match key {
                Keycode::Left => {
                    self.accelerate(Vec2::new(-self.speed(), 0.0), elapsed);
                }
                Keycode::Right => {
                    self.accelerate(Vec2::new(self.speed(), 0.0), elapsed);
                }
                Keycode::Down => {
                    self.crouched = true;
                }
                Keycode::Space => {
                    if self.grounded() {
                        self.jump();
                        sounds.push(SoundEffect::Jump);
                    }
                }
                _ => {}
            }
        }

        // Jump higher if key is held
        if !keys.contains(&Keycode::Space) && self.velocity.y > 0.0 {
            self.velocity.y = self.velocity.y.min(JUMP_SPEED / 2.0);
        }

        // Gravity
        self.accelerate(physics::GRAVITY, elapsed);

        let mut displacement = self.velocity * elapsed;

        self.grounded = false;
        // Check for collisions
        for t in tiles {
            // Check X component
            let x_collision = physics::collides(
                self.position + displacement * Vec2::X,
                self.sides,
                t.position,
                t.sides,
            );

            if x_collision {
                displacement.x = 0.0;
                self.velocity.x = 0.0;
            }

            // Check Y component
            let y_collision = physics::collides(
                self.position + displacement * Vec2::Y,
                self.sides,
                t.position,
                t.sides,
            );

            if y_collision {
                // Going down
                if self.velocity.y < 0.0 {
                    self.grounded = true;
                }
                displacement.y = 0.0;
                self.velocity.y = 0.0;
            }
        }
        // Apply new Position
        self.position += displacement;

        let col: i32 = (self.timer.elapsed().as_millis() as i32 / 160 % 4) * 128;
        if !self.grounded() {
            self.sprite = (0, 512, 128, 256);
        } else if self.velocity.x.abs() > 0.0 {
            self.sprite = (col, 256, 128, 256);
        } else {
            self.sprite = (col, 0, 128, 256);
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
