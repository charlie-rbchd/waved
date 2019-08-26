#![allow(dead_code)]

use std::path::PathBuf;

#[derive(Default)]
pub struct AudioFile {
    pub filename: PathBuf,
    pub samples: Vec<i32>,
}

#[derive(Default)]
pub struct State {
    pub current_file: Option<AudioFile>,
}