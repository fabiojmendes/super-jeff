use std::fs;
use std::io;
use std::vec::Vec;

use glam::Vec2;

use crate::physics;

#[derive(Debug)]
pub struct Enemy {
    pub position: Vec2,
    pub start_pos: Vec2,
    pub side: Vec2,
    pub velocity: Vec2,
}

const TILE_SIDE: f32 = 1.0;

#[derive(Debug)]
pub struct Tile {
    pub position: Vec2,
    pub side: f32,
}

#[derive(Debug)]
pub struct Level {
    pub bounds:Vec2,
    pub tiles: Vec<Tile>,
    pub enemies: Vec<Enemy>,
    pub spawn: Vec2,
}

impl Level {
    pub fn update(&mut self, elapsed: f32) {
        for e in &mut self.enemies {
            let displacement = e.velocity * elapsed;

            let mut x_collision = false;
            let mut y_collision = false;
            for t in &self.tiles {
                if physics::collides(
                    e.position + displacement,
                    e.side.into(),
                    t.position,
                    (t.side, t.side),
                ) {
                    x_collision = true;
                    break;
                }

                let future_y_pos = e.position + (displacement * 15.0) + Vec2::new(0.0, -0.1);
                y_collision |=
                    physics::collides(future_y_pos, e.side.into(), t.position, (t.side, t.side));
            }
            if x_collision || !y_collision {
                e.velocity = -e.velocity
            }
            let displacement = e.velocity * elapsed;
            e.position += displacement;
        }
    }
}

impl Level {
    pub fn new() -> Level {
        Level { bounds: Vec2::ZERO, tiles: Vec::new(), enemies: Vec::new(), spawn: Vec2::ZERO }
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
                        side: TILE_SIDE,
                    });
                }
                'E' => {
                    let position = world_pos;
                    level.enemies.push(Enemy {
                        position: position,
                        start_pos: position,
                        velocity: Vec2::new(-5.0, 0.0),
                        side: Vec2::new(1.0, 2.0),
                    });
                }
                'S' => {
                    let position = world_pos;
                    level.spawn = position;
                }
                _ => {}
            }
        }

        Ok(level)
    }
}
