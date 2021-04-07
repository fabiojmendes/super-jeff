use rand::{self, Rng};
use std::time::{Duration, Instant};
use std::vec::Vec;

use glam::Vec2;

use crate::level::Tile;
use crate::physics;

#[derive(Debug)]
pub struct Monkey {
    pub position: Vec2,
    pub sides: Vec2,
    pub velocity: Vec2,
    pub bananas: Vec<Banana>,
    pub enranged: bool,
    timer: Instant,
    bananas_thrown: i32,
    rage_velocity: Vec2,
    health: i32,
}

impl Monkey {
    const RAGE_DELAY: Duration = Duration::from_millis(750);
    const BANANA_MAX_DISTANCE: f32 = 30.0;

    pub fn new() -> Monkey {
        Monkey {
            position: Vec2::ZERO,
            sides: Vec2::new(2.0, 3.5),
            velocity: Vec2::ZERO,
            bananas: Vec::new(),
            bananas_thrown: 0,
            timer: Instant::now(),
            enranged: false,
            rage_velocity: Vec2::new(-15.0, 0.0),
            health: 3,
        }
    }

    fn rage(&mut self) {
        self.enranged = true;
        self.bananas_thrown = 0;
    }

    fn throw_banana(&mut self, target: Vec2) {
        let displacement = target - self.position;
        if displacement.x.abs() > Monkey::BANANA_MAX_DISTANCE {
            return;
        }
        // Random y velocity based on the distance from the target
        let yvel = (rand::random::<f32>() * 7.5) + (displacement.x.abs() / 2.0);
        // Calculate the trajectory based on the random y velocity and distance from target
        // https://www.dummies.com/education/science/physics/calculate-the-range-of-a-projectile-fired-at-an-angle/
        let velocity = Vec2::new((displacement.x * -physics::GRAVITY.y / yvel) / 2.0, yvel);
        self.bananas.push(Banana { position: self.position, sides: Vec2::new(0.5, 0.3), velocity });
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
        } else if self.enranged && self.timer.elapsed() >= Monkey::RAGE_DELAY {
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
        } else if self.bananas_thrown >= rng.gen_range(5..10) {
            self.rage();
        } else if self.timer.elapsed() > Duration::from_millis(rng.gen_range(1000..2000)) {
            self.timer += self.timer.elapsed();
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
