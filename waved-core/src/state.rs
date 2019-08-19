use std::path::PathBuf;

#[allow(dead_code)]
#[derive(Default)]
pub struct State {
    pub current_file: Option<PathBuf>,
}
