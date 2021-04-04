use std::fs;
use std::io;
use std::vec::Vec;

use glam::Vec2;

#[derive(Debug)]
pub struct Enemy {
    pub position: Vec2,
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
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<Tile>,
    pub enemies: Vec<Enemy>,
}

impl Level {
    pub fn update(&mut self, elapsed: f32) {
        for e in &mut self.enemies {
            e.velocity = Vec2::new(-5.0, 0.0);

            e.position += e.velocity * elapsed;

        }
    }
}

impl Level {
    pub fn new() -> Level {
        Level { width: 0, height: 0, tiles: Vec::new(), enemies: Vec::new() }
    }

    pub fn from_file(filename: &str, world_size: (f32, f32)) -> io::Result<Level> {
        let tile_offset = TILE_SIDE / 2.0;
        let (x_offset, y_offset) = (world_size.0 / 2.0, world_size.1 / 2.0);

        let level_str = fs::read_to_string(filename)?;

        let mut level = Level::new();

        level_str
            .lines()
            .enumerate()
            .flat_map(|(y, line)| line.char_indices().map(move |(x, c)| (x, y, c)))
            .for_each(|(x, y, c)| {
                level.width = level.width.max(x as i32 + 1);
                level.height = level.height.max(y as i32 + 1);
                let (tx, ty) =
                    (x as f32 - x_offset + tile_offset, -(y as f32) + y_offset - tile_offset);
                match c {
                    '#' => {
                        level.tiles.push(Tile { position: Vec2::new(tx, ty), side: TILE_SIDE });
                    }
                    'E' => {
                        level.enemies.push(Enemy {
                            position: Vec2::new(x as f32 - x_offset + 0.5, -(y as f32) + y_offset),
                            velocity: Vec2::new(0.0, 0.0),
                            side: Vec2::new(1.0, 2.0),
                        });
                    }
                    _ => {}
                }
            });

        Ok(level)
    }
}
