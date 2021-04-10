#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod level;
mod monkey;
mod physics;
mod player;
mod render;

use level::Level;
use render::Camera;

use glam::Vec2;
use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use std::env;
use std::time::Instant;

const FIXED_TIMESTEP: f32 = 1.0 / 60.0;

fn main() -> Result<(), String> {
    let fixed = env::args().find(|arg| arg == "--fixed").is_some();
    if fixed {
        println!("Using fixed timestep: {}", FIXED_TIMESTEP);
    }

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Super Jeff", 1280, 720)
        .position_centered()
        .build()
        .expect("could not build video subsystem");

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .expect("could not build canvas from window, quiting");

    let texture_creator = canvas.texture_creator();

    // TODO: Figure a better way to handle textures
    let textures = vec![
        texture_creator.load_texture("assets/jeff.png")?,
        texture_creator.load_texture("assets/monkey.png")?,
        texture_creator.load_texture("assets/banana.png")?,
        texture_creator.load_texture("assets/andi.png")?,
        texture_creator.load_texture("assets/leandro.png")?,
        texture_creator.load_texture("assets/paulo.png")?,
        texture_creator.load_texture("assets/vereador.png")?,
        texture_creator.load_texture("assets/newton.png")?,
        texture_creator.load_texture("assets/be-pimp.png")?,
        texture_creator.load_texture("assets/gold.png")?,
        texture_creator.load_texture("assets/lopes.png")?,
        texture_creator.load_texture("assets/ronald.png")?,
    ];

    let mut level = Level::from_file("assets/level.txt") //
        .expect("Error loading level from file");

    let mut camera = Camera::new(canvas.output_size()?);

    let mut timer = Instant::now();
    let mut event_pump = sdl_context.event_pump()?;
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                }
                Event::KeyDown { keycode: Some(Keycode::R), .. } => {
                    level = Level::from_file("assets/level.txt") //
                        .expect("Error loading level from file");
                }
                _ => {}
            }
        }
        let elapsed = if fixed {
            FIXED_TIMESTEP
        } else {
            let delta = timer.elapsed();
            timer += delta;
            delta.as_millis() as f32 / 1000.0
        };

        // Create a set of pressed Keys.
        let keys = event_pump
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();

        if !level.player.dead {
            level.update(elapsed, &keys);
        }

        if level.trapped {
            let bottom_right = Vec2::new(level.max_bounds().x, level.min_bounds().y);
            camera.recenter(bottom_right, level.max_bounds());
        } else {
            camera.recenter(level.player.position, level.max_bounds());
        }

        render::render(&mut canvas, &camera, &level, &textures)?;
    }

    Ok(())
}
