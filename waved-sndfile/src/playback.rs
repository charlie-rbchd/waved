use std::thread;
use std::time::Duration;
use std::iter::Iterator;

use cpal::{self, StreamData, UnknownTypeOutputBuffer, SampleFormat};
use cpal::traits::{HostTrait, DeviceTrait, EventLoopTrait};

use ringbuf::{self, RingBuffer};

fn sine_generator(sample_rate: u32) -> impl std::iter::Iterator<Item = f32> {
    let mut t = 0.0;
    let t_inc = 1.0 / sample_rate as f32;

    let f = 1000.0;
    let w = 2.0 * std::f32::consts::PI * f;

    std::iter::from_fn(move || {
        let s = (w * t as f32).sin();
        t += t_inc;
        Some(s)
    })
}

struct SignalChain {

}

pub fn create_audio_thread(buffer_size: usize) {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device available.");

    // TODO: We may need to support other sample formats depending on what
    // the target systems provide.
    let mut supported_formats_range = device.supported_output_formats()
        .expect("Error while querying formats.");
    let format = supported_formats_range.find(|f| f.data_type == SampleFormat::F32)
        .expect("No supported device format.")
        .with_max_sample_rate();
    let channels = format.channels;
    let sample_rate = format.sample_rate.0;

    // TODO: Can we know the size of the underlying device buffer?
    let ring = RingBuffer::<f32>::new(buffer_size);
    let (mut producer, mut consumer) = ring.split();

    // Spawn audio mixing thread.
    thread::spawn(move || {
        // TODO: Perform mixing according to a signal chain state
        let t_sleep = buffer_size as f32 / sample_rate as f32 * 0.5;
        let mut sine = sine_generator(sample_rate);
        loop {
            while !producer.is_full() {
                producer.push(sine.next().unwrap()).unwrap();
            }
            thread::sleep(Duration::from_secs_f32(t_sleep));
        }
    });

    // Spawn audio device thread.
    thread::spawn(move || {
        let event_loop = host.event_loop();
        let stream_id = event_loop.build_output_stream(&device, &format).unwrap();
        event_loop.play_stream(stream_id).expect("Failed to play_stream.");
        event_loop.run(move |stream_id, stream_result| {
            let stream_data = match stream_result {
                Ok(data) => data,
                Err(err) => {
                    eprintln!("An error occurred on stream {:?}: {}", stream_id, err);
                    return;
                },
            };

            match stream_data {
                StreamData::Output { buffer: UnknownTypeOutputBuffer::F32(mut buffer) } => {
                    for (i, elem) in buffer.iter_mut().enumerate() {
                        if i % channels as usize == 0 {
                            *elem = consumer.pop().unwrap_or(0.0);
                        } else {
                            *elem = 0.0;
                        }
                    }
                },
                _ => unreachable!(),
            }
        });
    });
}
