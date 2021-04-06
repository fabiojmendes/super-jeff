use rand;
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
    timer: Instant,
}

impl Monkey {
    fn new() -> Monkey {
        Monkey {
            position: Vec2::ZERO,
            sides: Vec2::new(2.0, 4.0),
            velocity: Vec2::ZERO,
            bananas: Vec::new(),
            timer: Instant::now(),
        }
    }

    fn udpate(&mut self, elapsed: f32, target: Vec2) {
        if self.timer.elapsed() > Duration::from_secs(1) {
            self.timer += self.timer.elapsed();
            let yvel = rand::random::<f32>() * 15.0 + 5.0;

            let distance = target - self.position;
            let velocity = Vec2::new((distance.x * -physics::GRAVITY.y / yvel) / 2.0, yvel);

            self.bananas.push(Banana {
                position: self.position,
                sides: Vec2::new(0.5, 0.2),
                velocity,
            })
        }
        for b in &mut self.bananas {
            b.velocity += physics::GRAVITY * elapsed;
            b.position += b.velocity * elapsed;
        }
        self.bananas.retain(|b| b.position.y > -20.0);
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
    pub start_pos: Vec2,
    pub sides: Vec2,
    pub velocity: Vec2,
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

        self.monkey.udpate(elapsed, player_pos);
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
            let world_pos = Vec2::new(x as f32 - offset.x, -(y as f32) + offset.y);
            match c {
                '#' => {
                    level.tiles.push(Tile {
                        position: world_pos + tile_offset,
                        sides: Vec2::new(TILE_SIDE, TILE_SIDE),
                    });
                }
                'E' => {
                    level.enemies.push(Enemy {
                        position: world_pos,
                        start_pos: world_pos,
                        velocity: Vec2::new(-5.0, 0.0),
                        sides: Vec2::new(1.0, 2.0),
                    });
                }
                'M' => {
                    level.monkey.position = world_pos + Vec2::Y;
                }
                'S' => {
                    level.spawn = world_pos;
                }
                _ => {}
            }
        }

        Ok(level)
    }
}
