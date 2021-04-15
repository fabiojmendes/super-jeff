use sdl2::mixer::{self, Chunk};
use std::collections::HashMap;

#[derive(Eq, PartialEq, Hash)]
pub enum SoundEffect {
    Jump,
    Hit,
    Click,
    Dead,
    Fall,
    Banana,
    Rage,
}

pub struct Sound<'a> {
    sound_registry: HashMap<SoundEffect, Chunk>,
    music: mixer::Music<'a>,
}

impl<'a> Sound<'a> {
    pub fn load() -> Result<Sound<'a>, String> {
        let frequency = 44_100;
        let format = mixer::AUDIO_S16LSB; // signed 16 bit samples, in little-endian byte order
        let channels = mixer::DEFAULT_CHANNELS; // Stereo
        let chunk_size = 1024;
        sdl2::mixer::open_audio(frequency, format, channels, chunk_size)?;
        mixer::allocate_channels(8);

        let mut sound_registry: HashMap<SoundEffect, Chunk> = HashMap::new();

        use SoundEffect::*;
        sound_registry.insert(Jump, Chunk::from_file("assets/jump.wav")?);
        sound_registry.insert(Hit, Chunk::from_file("assets/hit.wav")?);
        sound_registry.insert(Click, Chunk::from_file("assets/click.wav")?);
        sound_registry.insert(Dead, Chunk::from_file("assets/dead.wav")?);
        sound_registry.insert(Fall, Chunk::from_file("assets/fall.wav")?);
        sound_registry.insert(Banana, Chunk::from_file("assets/banana.wav")?);
        sound_registry.insert(Rage, Chunk::from_file("assets/rage.wav")?);

        let music = mixer::Music::from_file("assets/music.ogg")?;
        mixer::Music::set_volume(24);

        Ok(Sound { sound_registry, music })
    }

    pub fn play_music(&self) -> Result<(), String> {
        self.music.play(-1)
    }

    pub fn play_sounds(&self, sounds: Vec<SoundEffect>) {
        for s in &sounds {
            if let Some(s) = self.sound_registry.get(s) {
                if let Err(e) = mixer::Channel::all().play(s, 0) {
                    println!("Error playing sound: {}", e);
                }
            }
        }
    }
}
