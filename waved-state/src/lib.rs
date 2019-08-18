use hound::{WavReader, WavSamples, WavWriter};

use std::io::{BufReader, BufWriter};
use std::fs::File;
use std::path::Path;

#[allow(dead_code)]
#[derive(Default)]
pub struct State {
    reader: Option<WavReader<BufReader<File>>>,
    writer: Option<WavWriter<BufWriter<File>>>,
}

impl State {
    pub fn set_reader<P: AsRef<Path>>(&mut self, filename: P) {
        self.reader = Some(WavReader::open(filename).unwrap());
    }

    // TODO: Return samples here and use them to draw a waveform
    // pub fn samples(&self) -> Option<WavSamples<BufReader<File>, i16>> {
    //     self.reader.map_or(None, |r| Some(r.samples::<i16>()))
    // }
}
