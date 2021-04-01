use std::fs::File;
use std::io;
use std::io::prelude::*;

#[derive(Debug)]
pub struct Level {
    pub width: i32,
    pub height: i32,
    pub data: String,
}

impl Level {
    pub fn new(filename: &str) -> io::Result<Level> {
        let file = File::open(filename)?;
        let reader = io::BufReader::new(file);

        let mut data = String::new();
        let mut width = 0;
        let mut height = 0;

        for line in reader.lines() {
            let line = line?;
            if width == 0 {
                width = line.len() as i32;
            }
            height += 1;
            data += &line;
        }

        Ok(Level { width, height, data })
    }
}
