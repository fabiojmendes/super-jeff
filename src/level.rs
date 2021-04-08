use std::collections::HashSet;
use std::fs;
use std::io;
use std::time::Instant;
use std::vec::Vec;

use glam::{const_vec2, Vec2};
use sdl2::keyboard::Keycode;

use crate::monkey::Monkey;
use crate::physics;
use crate::player::Player;

#[derive(Debug)]
pub struct Enemy {
    pub spawn: Vec2,
    pub position: Vec2,
    pub sides: Vec2,
    pub velocity: Vec2,
    health: i32,
    pub sprite: (i32, i32, u32, u32),
    timer: Instant,
}

impl Enemy {
    const INITIAL_HEALTH: i32 = 1;
    const INITIAL_VELOCITY: Vec2 = const_vec2!([-5.0, 0.0]);

    pub fn new() -> Enemy {
        Enemy {
            spawn: Vec2::ZERO,
            position: Vec2::ZERO,
            sides: Vec2::new(1.5, 3.0),
            velocity: Enemy::INITIAL_VELOCITY,
            health: Enemy::INITIAL_HEALTH,
            sprite: (0, 0, 128, 256),
            timer: Instant::now(),
        }
    }

    pub fn dead(&self) -> bool {
        self.health <= 0
    }

    pub fn head(&self) -> (Vec2, Vec2) {
        let head = Vec2::new(self.position.x, self.position.y + self.sides.y / 2.0);
        (head, Vec2::new(self.sides.x, 0.2))
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

        if self.velocity.x.abs() > 0.0 {
            let col: i32 = (self.timer.elapsed().as_millis() as i32 / 160 % 4) * 128;
            self.sprite = (col, 0, 128, 256);
        } else {
            self.sprite = (0, 0, 128, 256);
        }
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
    pub monkey: Monkey,
    pub trapped: bool,
    trap_tiles: Vec<Tile>,
}

impl Level {
    pub fn new() -> Level {
        Level {
            bounds: Vec2::ZERO,
            tiles: Vec::new(),
            enemies: Vec::new(),
            monkey: Monkey::new(),
            player: Player::new(),
            trapped: false,
            trap_tiles: Vec::new(),
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

        if self.trap_tiles.iter().find(|t| self.player.position.x > t.position.x + 1.0).is_some() {
            self.tiles.append(&mut self.trap_tiles);
            self.trapped = true;
        }

        for e in &mut self.enemies {
            e.update(elapsed, &self.tiles);
        }

        // Player dies by falling out of level bounds
        if self.player.position.y < self.min_bounds().y - self.player.sides.y * 2.0 {
            self.player.die();
        }

        // Resolve Collisions
        if !self.monkey.dead() {
            let (head_pos, head_rect) = self.monkey.head();
            if self.player.attack(head_pos, head_rect) {
                self.monkey.damage(1);
            } else if physics::collides(
                self.player.position,
                self.player.sides,
                self.monkey.position,
                self.monkey.sides,
            ) {
                self.player.die();
            }
        }

        for b in &self.monkey.bananas {
            if physics::collides(self.player.position, self.player.sides, b.position, b.sides) {
                self.player.die();
            }
        }

        for e in self.enemies.iter_mut().filter(|e| !e.dead()) {
            let (head_pos, head_rect) = e.head();
            if self.player.attack(head_pos, head_rect) {
                e.damage(1);
            } else if physics::collides(
                self.player.position,
                self.player.sides,
                e.position,
                e.sides,
            ) {
                self.player.die();
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
                '@' => {
                    level
                        .trap_tiles
                        .push(Tile { position: world_pos, sides: Vec2::new(TILE_SIDE, TILE_SIDE) });
                }
                'E' => {
                    let mut e = Enemy::new();
                    e.spawn = Level::offset(world_pos, e.sides.y);
                    e.position = e.spawn;
                    level.enemies.push(e);
                }
                'M' => {
                    level.monkey.spawn = Level::offset(world_pos, level.monkey.sides.y);
                    level.monkey.position = level.monkey.spawn
                }
                'S' => {
                    level.player.spawn = Level::offset(world_pos, level.player.sides.y);
                    level.player.position = level.player.spawn;
                }
                _ => {}
            }
        }

        Ok(level)
    }
}
