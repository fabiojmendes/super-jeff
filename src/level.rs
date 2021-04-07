use std::collections::HashSet;
use std::fs;
use std::io;
use std::vec::Vec;

use glam::Vec2;
use sdl2::keyboard::Keycode;

use crate::monkey::Monkey;
use crate::physics;
use crate::player::Player;

#[derive(Debug)]
pub struct Enemy {
    pub position: Vec2,
    pub sides: Vec2,
    velocity: Vec2,
    health: i32,
}

impl Enemy {
    pub fn dead(&self) -> bool {
        self.health <= 0
    }

    pub fn damage(&mut self, damage: i32) {
        self.health -= damage;
        if self.dead() {
            self.velocity = Vec2::ZERO;
        }
    }

    pub fn update(&mut self, elapsed: f32, tiles: &Vec<Tile>) {
        let displacement = self.velocity * elapsed;

        let mut x_collision = false;
        let mut y_collision = false;
        for t in tiles {
            if physics::collides(self.position + displacement, self.sides, t.position, t.sides) {
                x_collision = true;
                break;
            }

            let future_y_pos =
                self.position + (Vec2::X * displacement.signum()) + Vec2::new(0.0, -0.2);
            y_collision |= physics::collides(future_y_pos, self.sides, t.position, t.sides);
        }
        if x_collision || !y_collision {
            self.velocity = -self.velocity
        }
        let displacement = self.velocity * elapsed;
        self.position += displacement;
    }
}

const TILE_SIDE: f32 = 1.0;

#[derive(Debug)]
pub struct Tile {
    pub position: Vec2,
    pub sides: Vec2,
}

#[derive(Debug)]
pub struct Level {
    bounds: Vec2,
    pub tiles: Vec<Tile>,
    pub enemies: Vec<Enemy>,
    pub player: Player,
    spawn: Vec2,
    pub monkey: Monkey,
}

impl Level {
    pub fn new() -> Level {
        Level {
            bounds: Vec2::ZERO,
            tiles: Vec::new(),
            enemies: Vec::new(),
            monkey: Monkey::new(),
            player: Player::new(),
            spawn: Vec2::ZERO,
        }
    }

    pub fn min_bounds(&self) -> Vec2 {
        -self.max_bounds()
    }

    pub fn max_bounds(&self) -> Vec2 {
        Vec2::new(self.bounds.x / 2.0, self.bounds.y / 2.0)
    }

    pub fn update(&mut self, elapsed: f32, keys: &HashSet<Keycode>) {
        self.player.update(keys, elapsed, &self.tiles);

        self.monkey.udpate(elapsed, self.player.position, &self.tiles);

        for e in &mut self.enemies {
            e.update(elapsed, &self.tiles);
        }

        // Player dies by falling out of level bounds
        if self.player.position.y < self.min_bounds().y - self.player.sides.y * 2.0 {
            self.player.die(self.spawn);
        }

        // Resolve Collisions
        let (foot_pos, foot_rect) = self.player.foot_rect();

        if !self.monkey.dead() {
            if physics::collides(foot_pos, foot_rect, self.monkey.position, self.monkey.sides) {
                self.player.velocity.y = 15.0;
                self.player.position.y += 0.5;
                self.monkey.damage(1);
            } else if physics::collides(
                self.player.position,
                self.player.sides,
                self.monkey.position,
                self.monkey.sides,
            ) {
                self.player.die(self.spawn);
            }
        }

        for b in &self.monkey.bananas {
            if physics::collides(self.player.position, self.player.sides, b.position, b.sides) {
                self.player.die(self.spawn);
            }
        }

        for e in self.enemies.iter_mut().filter(|e| !e.dead()) {
            if physics::collides(foot_pos, foot_rect, e.position, e.sides) {
                e.damage(1);
            } else if physics::collides(
                self.player.position,
                self.player.sides,
                e.position,
                e.sides,
            ) {
                self.player.die(self.spawn);
            }
        }

        let min_bounds = self.min_bounds();
        self.monkey.bananas.retain(|b| b.position.y > min_bounds.y);
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
                        health: 1,
                    });
                }
                'M' => {
                    level.monkey.position = Level::offset(world_pos, level.monkey.sides.y);
                }
                'S' => {
                    level.spawn = Level::offset(world_pos, level.player.sides.y);
                    level.player.position = level.spawn;
                }
                _ => {}
            }
        }

        Ok(level)
    }
}
