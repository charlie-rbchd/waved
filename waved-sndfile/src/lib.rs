use hound::WavReader;
pub use hound::{Error, Sample, SampleFormat};

use std::path::Path;

pub fn samples_from_file<P: AsRef<Path>>(filename: P) -> Result<Vec<f32>, Error> {
    // TODO: Downmix the channels instead of filtering them out
    let mut reader = WavReader::open(filename)?;
    let spec = reader.spec();
    let samples = match spec.sample_format {
        SampleFormat::Float => {
            reader.samples::<f32>()
                .enumerate()
                .filter(|(i, _)| i % spec.channels as usize == 0)
                .map(|(_, s)| s.unwrap())
                .collect()
        },
        SampleFormat::Int => {
            // The range values assume the integers are encoded using two's complement
            let neg_range = 2i32.pow((spec.bits_per_sample - 1).into());
            let pos_range = neg_range - 1;
            reader.samples::<i32>()
                .enumerate()
                .filter(|(i, _)| i % spec.channels as usize == 0)
                .map(|(_, s)| {
                    let s = s.unwrap();
                    if s < 0 {
                        (s as f64 / neg_range as f64) as f32
                    } else {
                        (s as f64 / pos_range as f64) as f32
                    }
                })
                .collect()
        },
    };
    Ok(samples)
}
