#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod level;
mod physics;
mod player;
mod render;

use level::Level;
use player::Player;
use render::Camera;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Instant;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Super Jeff", 1024, 768)
        .position_centered()
        .build()
        .expect("could not build video subsystem");

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .expect("could not build canvas from window, quiting");

    let mut timer = Instant::now();

    let mut level = Level::from_file("level.txt") //
        .expect("Error loading level from file");

    let mut player = Player::new(level.spawn);

    let mut camera = Camera::new(canvas.output_size()?);

    let mut event_pump = sdl_context.event_pump()?;
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                }
                _ => {}
            }
        }
        let delta = timer.elapsed();
        let elapsed = delta.as_millis() as f32 / 1000.0;
        timer += delta;

        // Create a set of pressed Keys.
        let keys = event_pump
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();

        player.update(&keys, elapsed, &mut level);

        level.update(elapsed, player.position);

        camera.recenter(player.position, level.bounds);

        render::render(&mut canvas, &camera, &player, &level)?;
    }

    Ok(())
}
