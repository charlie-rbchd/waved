use hound::WavReader;
pub use hound::{Error, Sample, SampleFormat};
use itertools::Itertools;

use std::path::Path;

pub fn samples_from_file<P: AsRef<Path>>(filename: P) -> Result<Vec<Vec<f32>>, Error> {
    // FIXME: It seems like samples are collected correctly when there are multiple channels
    let mut reader = WavReader::open(filename)?;
    let spec = reader.spec();
    match spec.sample_format {
        SampleFormat::Float => {
            Ok(reader.samples::<f32>()
                .enumerate()
                .group_by(|(i, _)| i % spec.channels as usize)
                .into_iter()
                .map(|(_, chan)| chan.map(|(_, s)| s.unwrap()).collect())
                .collect())
        },
        SampleFormat::Int => {
            // The range values assume the integers are encoded using two's complement
            let neg_range = 2i32.pow((spec.bits_per_sample - 1).into());
            let pos_range = neg_range - 1;
            Ok(reader.samples::<i32>()
                .enumerate()
                .group_by(|(i, _)| i % spec.channels as usize)
                .into_iter()
                .map(|(_, chan)| chan.map(|(_, s)| {
                    let s = s.unwrap();
                    if s < 0 {
                        (s as f64 / neg_range as f64) as f32
                    } else {
                        (s as f64 / pos_range as f64) as f32
                    }
                }).collect())
                .collect())
        },
    }
}
