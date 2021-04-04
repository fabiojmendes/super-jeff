use std::fs;
use std::io;
use std::vec::Vec;

use glam::Vec2;

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
}

impl Level {
    pub fn new() -> Level {
        Level { width: 0, height: 0, tiles: Vec::new() }
    }

    pub fn from_file(filename: &str, world_size: (f32, f32)) -> io::Result<Level> {
        let tile_offset = TILE_SIDE / 2.0;
        let (x_offset, y_offset) = (world_size.0 / 2.0, world_size.1 / 2.0);

        let level_str = fs::read_to_string(filename)?;

        let level = level_str
            .lines()
            .enumerate()
            .flat_map(|(y, line)| line.char_indices().map(move |(x, c)| (x, y, c)))
            .filter_map(|(x, y, c)| {
                let (tx, ty) =
                    (x as f32 - x_offset + tile_offset, -(y as f32) + y_offset - tile_offset);
                match c {
                    '#' => Some((x, y, Tile { position: Vec2::new(tx, ty), side: TILE_SIDE })),
                    _ => None,
                }
            })
            .fold(Level::new(), |level, (x, y, tile)| {
                let mut level = level;
                level.width = level.width.max(x as i32);
                level.height = level.height.max(y as i32);
                level.tiles.push(tile);
                level
            });

        Ok(level)
    }
}
