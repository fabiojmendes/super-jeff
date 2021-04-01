use std::fs::File;
use std::io;
use std::io::prelude::*;
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
    pub fn new(filename: &str, world_size: (f32, f32)) -> io::Result<Level> {
        let file = File::open(filename)?;
        let reader = io::BufReader::new(file);

        let mut width = 0;
        let mut height = 0;

        let tile_offset = TILE_SIDE / 2.0;
        let (x_offset, y_offset) = (world_size.0 / 2.0, world_size.1 / 2.0);

        let mut tiles = Vec::new();
        for (y, line) in reader.lines().filter_map(|l| l.ok()).enumerate() {
            for (x, c) in line.chars().enumerate() {
                match c {
                    '#' => {
                        let (x, y) = (
                            x as f32 - x_offset + tile_offset,
                            -(y as f32) + y_offset - tile_offset,
                        );
                        tiles.push(Tile { position: Vec2::new(x, y), side: TILE_SIDE });
                    }
                    _ => {}
                }
            }
            if width == 0 {
                width = line.len() as i32;
            }
            height += 1;
        }

        Ok(Level { width, height, tiles })
    }
}
