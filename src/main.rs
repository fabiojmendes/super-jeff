#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod level;
mod monkey;
mod physics;
mod player;
mod render;

use level::Level;
use render::Camera;
use render::{TextManager, TextureManager};

use glam::Vec2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mixer::{self, Chunk};
use sdl2::ttf;
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
    let tx_manager = TextureManager::load(&texture_creator)?;

    // Font Subsystem
    let ttf_context = ttf::init().map_err(|e| e.to_string())?;
    // Load a font
    let font32 = ttf_context.load_font("assets/cocogoose.ttf", 32)?;
    let font64 = ttf_context.load_font("assets/cocogoose.ttf", 64)?;

    let text_manager = TextManager { texture_creator: &texture_creator, font32, font64 };
    let mut camera = Camera::new(canvas.output_size()?);

    // Audio Subsystem
    let _audio = sdl_context.audio()?;
    let frequency = 44_100;
    let format = mixer::AUDIO_S16LSB; // signed 16 bit samples, in little-endian byte order
    let channels = mixer::DEFAULT_CHANNELS; // Stereo
    let chunk_size = 1024;
    sdl2::mixer::open_audio(frequency, format, channels, chunk_size)?;
    mixer::allocate_channels(8);

    let jump = Chunk::from_file("assets/jump.wav")?;
    let hit = Chunk::from_file("assets/hit.wav")?;
    let click = Chunk::from_file("assets/click.wav")?;
    let dead = Chunk::from_file("assets/dead.wav")?;
    let fall = Chunk::from_file("assets/fall.wav")?;
    let banana = Chunk::from_file("assets/banana.wav")?;
    let rage = Chunk::from_file("assets/rage.wav")?;

    let music = mixer::Music::from_file("assets/music.mp3")?;
    mixer::Music::set_volume(24);
    music.play(-1)?;

    let mut level = Level::from_file("assets/level.txt") //
        .expect("Error loading level from file");

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
                    level.start();
                }
                Event::KeyUp { keycode: Some(Keycode::Space), .. } if !level.started() => {
                    level.start();
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

        // Create a set of pressed Keys
        let keys = event_pump
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();

        let mut sounds = Vec::<&str>::new();
        level.update(elapsed, &keys, &mut sounds);

        for s in sounds {
            let sound = match s {
                "jump" => Some(&jump),
                "hit" => Some(&hit),
                "click" => Some(&click),
                "dead" => Some(&dead),
                "fall" => Some(&fall),
                "banana" => Some(&banana),
                "rage" => Some(&rage),
                _ => None,
            };
            if let Some(s) = sound {
                if let Err(e) = mixer::Channel::all().play(s, 0) {
                    println!("Error playing sound: {}", e);
                }
            }
        }

        if level.trapped {
            let bottom_right = Vec2::new(level.max_bounds().x, level.min_bounds().y);
            camera.recenter(bottom_right, level.max_bounds());
        } else {
            camera.recenter(level.player.position, level.max_bounds());
        }

        render::render(&mut canvas, &camera, &level, &tx_manager, &text_manager)?;
    }

    Ok(())
}
