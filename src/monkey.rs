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
    ai_timer: Instant,
    rage_velocity: Vec2,
    health: i32,
    pub right: bool,
    pub sprite: (i32, i32, u32, u32),
    anim_timer: i32,
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
            ai_timer: Instant::now(),
            enranged: false,
            rage_velocity: Vec2::new(-15.0, 0.0),
            health: Monkey::INITIAL_HEALTH,
            right: true,
            sprite: (0, 0, 256, 256),
            anim_timer: 0,
        }
    }

    pub fn right(&self) -> bool {
        if self.enranged {
            self.rage_velocity.x < 0.0
        } else {
            self.right
        }
    }

    fn rage(&mut self) {
        self.enranged = true;
        self.bananas_thrown = 0;
    }

    fn throw_banana(&mut self, displacement: Vec2) {
        // Random y velocity based on current health the distance from the target
        let yvel =
            (rand::random::<f32>() * 4.0 + 2.0 * self.health as f32) + (displacement.x.abs() / 4.0);

        // Calculate the trajectory based on the random y velocity and distance from target
        // https://www.dummies.com/education/science/physics/calculate-the-range-of-a-projectile-fired-at-an-angle/
        let velocity = Vec2::new(((displacement.x * -physics::GRAVITY.y) / yvel) / 2.0, yvel);
        let position = self.position
            + Vec2::new(self.sides.x / 2.0 * displacement.x.signum(), -self.sides.y / 4.0);
        self.bananas.push(Banana { position, sides: Vec2::new(0.8, 0.4), velocity });
        self.bananas_thrown += 1;
    }

    pub fn dead(&self) -> bool {
        self.health <= 0
    }

    pub fn damage(&mut self, amount: i32) -> bool {
        if self.enranged {
            return false;
        }
        self.health -= amount;
        self.rage_velocity.x += 5.0 * self.rage_velocity.x.signum();
        if self.dead() {
            self.velocity = Vec2::ZERO;
        } else {
            self.rage();
        }
        true
    }

    pub fn head(&self) -> (Vec2, Vec2) {
        let head = Vec2::new(self.position.x, self.position.y + self.sides.y / 2.25);
        (head, Vec2::new(self.sides.x, 0.5))
    }

    pub fn hitbox(&self) -> Vec2 {
        self.sides - Vec2::new(0.25, 0.5)
    }

    pub fn udpate(
        &mut self,
        elapsed: f32,
        target: Vec2,
        tiles: &Vec<Tile>,
        sounds: &mut Vec<&str>,
    ) {
        let mut rng = rand::thread_rng();
        if self.dead() {
            // Skip
        } else if self.enranged
            && self.ai_timer.elapsed() >= Monkey::RAGE_DELAY * self.health as u32
        {
            self.velocity = self.rage_velocity;
            for t in tiles {
                let displacement = self.velocity.signum() * Vec2::X / 2.0;
                if physics::collides(self.position + displacement, self.sides, t.position, t.sides)
                {
                    self.ai_timer += self.ai_timer.elapsed();
                    self.enranged = false;
                    self.velocity = Vec2::ZERO;
                    self.rage_velocity = -self.rage_velocity;
                    break;
                }
            }
        } else if self.bananas_thrown >= self.bananas_before_rage {
            self.rage();
            sounds.push("rage");
            self.bananas_before_rage = rng.gen_range(5..10);
        } else if self.ai_timer.elapsed() > self.next_throw {
            self.ai_timer += self.ai_timer.elapsed();
            self.next_throw = Duration::from_millis(rng.gen_range(1000..2000));
            self.anim_timer = 0;
            let displacement = target - self.position;
            if displacement.x.abs() < Monkey::BANANA_MAX_DISTANCE {
                self.throw_banana(displacement);
                sounds.push("banana");
            }
        }

        self.position += self.velocity * elapsed;

        match self.next_throw.checked_sub(self.ai_timer.elapsed()) {
            Some(d) if d <= Duration::from_millis(50 * 4) && !self.enranged => {
                let col = (self.anim_timer / 50 % 4) * 128;
                self.sprite = (col, 0, 128, 256);
                self.anim_timer += (elapsed * 1000.0) as i32;
            }
            _ if self.enranged => {
                self.sprite = (4 * 128, 0, 128, 256);
            }
            _ => {
                self.sprite = (0, 0, 128, 256);
            }
        }

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
