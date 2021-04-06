use glam::{const_vec2, Vec2};
use sdl2::keyboard::Keycode;
use std::collections::HashSet;

use crate::level::Level;
use crate::physics;

const PLAYER_SPEED: f32 = 30.0;
const JUMP_SPEED: f32 = 15.0;

const MAX_VELOCITY: Vec2 = const_vec2!([10.0, 100.0]);

#[derive(Debug)]
pub struct Player {
    pub position: Vec2,
    sides: Vec2,
    velocity: Vec2,
    grounded: bool,
    crouched: bool,
}

impl Player {
    pub fn new(spawn: Vec2) -> Player {
        Player {
            position: spawn,
            sides: Vec2::new(0.9, 1.8),
            velocity: Vec2::ZERO,
            grounded: false,
            crouched: false,
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

    fn die(&mut self, spawn: Vec2) {
        self.velocity = Vec2::ZERO;
        self.position = spawn;
    }

    pub fn foot_rect(&self) -> (Vec2, Vec2) {
        let foot = Vec2::new(self.position.x, self.position.y - self.sides.y / 2.0 - 0.08);
        (foot, Vec2::new(0.55, 0.05))
    }

    pub fn update(&mut self, keys: &HashSet<Keycode>, elapsed: f32, level: &mut Level) {
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
                        self.velocity.y = JUMP_SPEED;
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
        for t in &level.tiles {
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

        let (foot_pos, foot_rect) = self.foot_rect();
        level.enemies.retain(|e| !physics::collides(foot_pos, foot_rect, e.position, e.sides));

        for e in &level.enemies {
            if physics::collides(self.position, self.sides, e.position, e.sides) {
                self.die(level.spawn);
            }
        }

        if physics::collides(self.position, self.sides, level.monkey.position, level.monkey.sides) {
            self.die(level.spawn);
        }

        for b in &level.monkey.bananas {
            if physics::collides(self.position, self.sides, b.position, b.sides) {
                self.die(level.spawn);
            }
        }

        // Reset if it falls
        if self.position.y < -(level.bounds.y) {
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
