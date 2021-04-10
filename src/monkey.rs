use rand::{self, Rng};
use std::time::{Duration, Instant};
use std::vec::Vec;

use glam::Vec2;

use crate::level::Tile;
use crate::physics;

#[derive(Debug)]
pub struct Monkey {
    pub spawn: Vec2,
    pub position: Vec2,
    pub sides: Vec2,
    pub velocity: Vec2,
    pub bananas: Vec<Banana>,
    next_throw: Duration,
    bananas_thrown: i32,
    bananas_before_rage: i32,
    pub enranged: bool,
    timer: Instant,
    rage_velocity: Vec2,
    health: i32,
    pub right: bool,
    pub sprite: (i32, i32, u32, u32),
}

impl Monkey {
    const RAGE_DELAY: Duration = Duration::from_millis(250);
    const BANANA_MAX_DISTANCE: f32 = 30.0;
    const INITIAL_HEALTH: i32 = 3;

    pub fn new() -> Monkey {
        Monkey {
            spawn: Vec2::ZERO,
            position: Vec2::ZERO,
            sides: Vec2::new(2.0, 4.0),
            velocity: Vec2::ZERO,
            bananas: Vec::new(),
            bananas_thrown: 0,
            bananas_before_rage: 7,
            next_throw: Duration::from_millis(1500),
            timer: Instant::now(),
            enranged: false,
            rage_velocity: Vec2::new(-15.0, 0.0),
            health: Monkey::INITIAL_HEALTH,
            right: true,
            sprite: (0, 0, 256, 256),
        }
    }

    pub fn right(&self) -> bool {
        if self.enranged {
            self.rage_velocity.x < 0.0
        } else {
            self.right
        }
    }

    pub fn head(&self) -> (Vec2, Vec2) {
        let head = Vec2::new(self.position.x, self.position.y + self.sides.y / 2.0);
        (head, Vec2::new(self.sides.x, 0.2))
    }

    fn rage(&mut self) {
        self.enranged = true;
        self.bananas_thrown = 0;
    }

    fn throw_banana(&mut self, target: Vec2) {
        let displacement = target - self.position;
        self.right = displacement.x < 0.0;
        if displacement.x.abs() > Monkey::BANANA_MAX_DISTANCE {
            return;
        }
        // Random y velocity based on current health the distance from the target
        let yvel =
            (rand::random::<f32>() * 4.0 + 2.0 * self.health as f32) + (displacement.x.abs() / 4.0);

        // Calculate the trajectory based on the random y velocity and distance from target
        // https://www.dummies.com/education/science/physics/calculate-the-range-of-a-projectile-fired-at-an-angle/
        let velocity = Vec2::new(((displacement.x * -physics::GRAVITY.y) / yvel) / 2.0, yvel);
        self.bananas.push(Banana { position: self.position, sides: Vec2::new(0.8, 0.4), velocity });
        self.bananas_thrown += 1;
    }

    pub fn dead(&self) -> bool {
        self.health <= 0
    }

    pub fn damage(&mut self, amount: i32) {
        if self.enranged {
            return;
        }
        self.health -= amount;
        self.rage_velocity.x += 5.0 * self.rage_velocity.x.signum();
        if self.dead() {
            self.velocity = Vec2::ZERO;
        } else {
            self.rage();
        }
    }

    pub fn udpate(&mut self, elapsed: f32, target: Vec2, tiles: &Vec<Tile>) {
        let mut rng = rand::thread_rng();
        if self.dead() {
            // Skip
        } else if self.enranged && self.timer.elapsed() >= Monkey::RAGE_DELAY * self.health as u32 {
            self.velocity = self.rage_velocity;
            for t in tiles {
                let displacement = self.velocity.signum() * Vec2::X;
                if physics::collides(self.position + displacement, self.sides, t.position, t.sides)
                {
                    self.timer += self.timer.elapsed();
                    self.enranged = false;
                    self.velocity = Vec2::ZERO;
                    self.rage_velocity = -self.rage_velocity;
                    break;
                }
            }
        } else if self.bananas_thrown >= self.bananas_before_rage {
            self.rage();
            self.bananas_before_rage = rng.gen_range(5..10);
        } else if self.timer.elapsed() > self.next_throw {
            self.timer += self.timer.elapsed();
            self.next_throw = Duration::from_millis(rng.gen_range(1000..2000));
            self.throw_banana(target);
        }

        self.position += self.velocity * elapsed;

        for b in &mut self.bananas {
            b.velocity += physics::GRAVITY * elapsed;
            b.position += b.velocity * elapsed;
        }
    }
}

#[derive(Debug)]
pub struct Banana {
    pub position: Vec2,
    pub sides: Vec2,
    velocity: Vec2,
}
