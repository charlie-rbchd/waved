#![allow(dead_code)]

use std::path::PathBuf;

pub struct AudioFile {
    pub filename: PathBuf,
    pub samples: Vec<f32>,
    pub num_channels: u16,
}

#[derive(Default)]
pub struct State {
    pub current_file: Option<AudioFile>,
}
