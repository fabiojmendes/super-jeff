use rand::{self, Rng};
use std::fs;
use std::io;
use std::time::{Duration, Instant};
use std::vec::Vec;

use glam::Vec2;

use crate::physics;

#[derive(Debug)]
pub struct Monkey {
    pub position: Vec2,
    pub sides: Vec2,
    pub velocity: Vec2,
    pub bananas: Vec<Banana>,
    bananas_thrown: i32,
    timer: Instant,
    pub enranged: bool,
    rage_velocity: Vec2,
}

impl Monkey {
    const RAGE_START: Duration = Duration::from_millis(500);
    const RAGE_DURATION: Duration = Duration::from_millis(2300);

    fn new() -> Monkey {
        Monkey {
            position: Vec2::ZERO,
            sides: Vec2::new(2.0, 3.5),
            velocity: Vec2::ZERO,
            bananas: Vec::new(),
            bananas_thrown: 0,
            timer: Instant::now(),
            enranged: false,
            rage_velocity: Vec2::new(-15.0, 0.0),
        }
    }

    fn rage(&mut self) {
        self.bananas_thrown = 0;
        self.enranged = true;
        println!("RAGE!");
    }

    fn throw_banana(&mut self, target: Vec2) {
        let distance = target - self.position;
        if distance.x.abs() > 30.0 {
            return;
        }
        // Random y velocity based on the distance from the target
        let yvel = (rand::random::<f32>() * 7.5) + (distance.x.abs() / 2.0);
        // Calculate the trajectory based on the random y velocity and distance from target
        // https://www.dummies.com/education/science/physics/calculate-the-range-of-a-projectile-fired-at-an-angle/
        let velocity = Vec2::new((distance.x * -physics::GRAVITY.y / yvel) / 2.0, yvel);
        self.bananas.push(Banana { position: self.position, sides: Vec2::new(0.5, 0.3), velocity });
        self.bananas_thrown += 1;
    }

    fn udpate(&mut self, elapsed: f32, target: Vec2, level_bounds: Vec2) {
        let mut rng = rand::thread_rng();
        if self.enranged {
            if self.timer.elapsed() >= Monkey::RAGE_START {
                self.velocity = self.rage_velocity;
            }
            if self.timer.elapsed() >= Monkey::RAGE_DURATION {
                self.timer += self.timer.elapsed();
                self.enranged = false;
                self.velocity = Vec2::ZERO;
                self.rage_velocity = -self.rage_velocity;
                println!("End Rage");
            }
        } else if self.bananas_thrown >= rng.gen_range(5..10) {
            self.rage();
        } else if self.timer.elapsed() > Duration::from_millis(rng.gen_range(500..1500)) {
            self.timer += self.timer.elapsed();
            self.throw_banana(target);
        }

        self.position += self.velocity * elapsed;

        for b in &mut self.bananas {
            b.velocity += physics::GRAVITY * elapsed;
            b.position += b.velocity * elapsed;
        }
        self.bananas.retain(|b| b.position.y > level_bounds.y);
    }
}

#[derive(Debug)]
pub struct Banana {
    pub position: Vec2,
    pub sides: Vec2,
    velocity: Vec2,
}

#[derive(Debug)]
pub struct Enemy {
    pub position: Vec2,
    pub sides: Vec2,
    velocity: Vec2,
}

const TILE_SIDE: f32 = 1.0;

#[derive(Debug)]
pub struct Tile {
    pub position: Vec2,
    pub sides: Vec2,
}

#[derive(Debug)]
pub struct Level {
    pub bounds: Vec2,
    pub tiles: Vec<Tile>,
    pub enemies: Vec<Enemy>,
    pub spawn: Vec2,
    pub monkey: Monkey,
}

impl Level {
    pub fn min_bounds(&self) -> Vec2 {
        -self.max_bounds()
    }

    pub fn max_bounds(&self) -> Vec2 {
        Vec2::new(self.bounds.x / 2.0, self.bounds.y / 2.0)
    }

    pub fn update(&mut self, elapsed: f32, player_pos: Vec2) {
        for e in &mut self.enemies {
            let displacement = e.velocity * elapsed;

            let mut x_collision = false;
            let mut y_collision = false;
            for t in &self.tiles {
                if physics::collides(e.position + displacement, e.sides, t.position, t.sides) {
                    x_collision = true;
                    break;
                }

                let future_y_pos =
                    e.position + (Vec2::X * displacement.signum()) + Vec2::new(0.0, -0.2);
                y_collision |= physics::collides(future_y_pos, e.sides, t.position, t.sides);
            }
            if x_collision || !y_collision {
                e.velocity = -e.velocity
            }
            let displacement = e.velocity * elapsed;
            e.position += displacement;
        }

        self.monkey.udpate(elapsed, player_pos, self.min_bounds());
    }
}

impl Level {
    pub fn new() -> Level {
        Level {
            bounds: Vec2::ZERO,
            tiles: Vec::new(),
            enemies: Vec::new(),
            monkey: Monkey::new(),
            spawn: Vec2::ZERO,
        }
    }

    fn offset(position: Vec2, y_side: f32) -> Vec2 {
        position + (Vec2::Y * (y_side - TILE_SIDE) / 2.0)
    }

    pub fn from_file(filename: &str) -> io::Result<Level> {
        let level_str = fs::read_to_string(filename)?;

        let mut level = Level::new();

        let level_coords: Vec<_> = level_str
            .lines()
            .enumerate()
            .flat_map(|(y, line)| line.char_indices().map(move |(x, c)| (x, y, c)))
            .collect();

        for (x, y, _) in &level_coords {
            level.bounds.x = level.bounds.x.max((*x + 1) as f32);
            level.bounds.y = level.bounds.y.max((*y + 1) as f32);
        }

        let tile_offset = Vec2::new(TILE_SIDE / 2.0, -TILE_SIDE / 2.0);
        let offset = level.bounds / 2.0;

        for (x, y, c) in level_coords {
            let world_pos = Vec2::new(x as f32 - offset.x, -(y as f32) + offset.y) + tile_offset;
            match c {
                '#' => {
                    level
                        .tiles
                        .push(Tile { position: world_pos, sides: Vec2::new(TILE_SIDE, TILE_SIDE) });
                }
                'E' => {
                    let sides = Vec2::new(1.0, 2.0);
                    level.enemies.push(Enemy {
                        position: Level::offset(world_pos, sides.y),
                        velocity: Vec2::new(-5.0, 0.0),
                        sides,
                    });
                }
                'M' => {
                    level.monkey.position = Level::offset(world_pos, level.monkey.sides.y);
                }
                'S' => {
                    // TODO: Get the sides value from player
                    level.spawn = Level::offset(world_pos, 1.8);
                }
                _ => {}
            }
        }

        Ok(level)
    }
}
