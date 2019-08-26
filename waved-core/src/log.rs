use std::error::Error;

pub struct Logger {
    messages: Vec<String>,
}

impl Logger {
    pub fn new() -> Self {
        Self { messages: vec![] }
    }

    pub fn log<E: Error>(&mut self, err: E) {
        self.messages.push(format!("{:?}", err));
    }
}
