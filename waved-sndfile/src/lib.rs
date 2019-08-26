use hound::WavReader;
pub use hound::{Error, Sample};

use std::path::Path;

pub fn samples_from_file<S: Sample, P: AsRef<Path>>(filename: P) -> Result<Vec<S>, Error> {
    let mut reader = WavReader::open(filename)?;
    let samples = reader.samples::<S>().collect();
    samples
}
