#![allow(dead_code)]

use nanovg::{Color, Context, ContextBuilder, Font, Frame, StrokeOptions};

use std::ops::Deref;

use waved_core::state::State;

pub struct Fonts<'f> {
    regular: Font<'f>,
    bold: Font<'f>,
}

pub struct Renderer<'f> {
    context: Box<Context>,
    fonts: Fonts<'f>,
}

fn draw_line(frame: &Frame, from: (f32, f32), to: (f32, f32)) {
    frame.path(|path| {
        path.move_to(from);
        path.line_to(to);
        path.stroke(
            Color::from_rgba(255, 255, 255, 255),
            StrokeOptions {
                width: 1.0,
                ..Default::default()
            }
        );
    }, Default::default());
}

fn draw_waveform(frame: &Frame, pos: (f32, f32), size: (f32, f32), samples: &[f32]) {
    let width = size.0;
    let height = size.1;
    let half_height  = height * 0.5;
    let top = pos.1;
    let left = pos.0;

    let draw_segment = |x: f32, weight: (f32, f32)| {
        let line_height = (half_height * weight.0, half_height * weight.1);
        let line_start_y = top + half_height - line_height.1;
        let line_end_y = top + half_height + line_height.0;
        draw_line(frame, (left + x, line_start_y), (left + x, line_end_y));
    };

    draw_line(frame, (left, top + half_height), (left + width, top + half_height));

    let mut current_x = 0;
    let mut sum_of_squares = (0.0, 0.0);
    let mut num_samples_acc = (0, 0);

    let num_samples = samples.len();
    for (i, s) in samples.iter().enumerate() {
        let x = (i as f32 / num_samples as f32 * width).round() as i32;
        if current_x == x {
            if *s < 0.0 {
                sum_of_squares.0 += s * s;
                num_samples_acc.0 += 1;
            } else {
                sum_of_squares.1 += s * s;
                num_samples_acc.1 += 1;
            }
        } else {
            draw_segment(x as f32, (
                (sum_of_squares.0 / num_samples_acc.0 as f32).sqrt(),
                (sum_of_squares.1 / num_samples_acc.1 as f32).sqrt()
            ));
            current_x = x;
            sum_of_squares = (0.0, 0.0);
            num_samples_acc = (0, 0);
        }
    }

    draw_segment(width, (
        (sum_of_squares.0 / num_samples_acc.0 as f32).sqrt(),
        (sum_of_squares.1 / num_samples_acc.1 as f32).sqrt()
    ));
}

impl<'f> Renderer<'f> {
    pub fn new() -> Self {
        // Has to be heap-allocated since we take it's address when creating fonts.
        let context = Box::new(ContextBuilder::new()
            .antialias()
            .stencil_strokes()
            .build()
            .expect("Failed to create a drawing context."));

        // Perform some unsafe pointer gymnastics to ignore lifetime constraints,
        // making it possible to store context and fonts in the same struct even though
        // context would normally have to outlive fonts since it is borrowed in
        // the call to Font::from_file.
        let fonts = {
            let context_ptr = context.deref() as *const _;
            Fonts {
                regular: Font::from_file(unsafe { &*context_ptr }, "Inconsolata-Regular", "resources/Inconsolata-Regular.ttf")
                    .expect("Failed to load font `Inconsolata-Regular.ttf`."),

                bold: Font::from_file(unsafe { &*context_ptr }, "Inconsolata-Bold", "resources/Inconsolata-Bold.ttf")
                    .expect("Failed to load font `Inconsolata-Bold.ttf`."),
            }
        };

        Self { context, fonts }
    }

    pub fn render(&self, state: &State, viewport: (f32, f32), scale: f32) {
        self.context.frame(viewport, scale, |frame| {
            if let Some(file) = &state.current_file {
                // TODO: Implement zoom and scroll
                let channel_height = viewport.1 / file.num_channels as f32;
                for i in 0..file.num_channels as usize {
                    let samples_per_channel = file.samples.len() / file.num_channels as usize;
                    let channel_slice_start = samples_per_channel * i;
                    let channel_slice_end = samples_per_channel * (i + 1);
                    draw_waveform(
                        &frame,
                        (0.0, i as f32 * channel_height),
                        (viewport.0, channel_height),
                        &file.samples[channel_slice_start..channel_slice_end]
                    );
                }
            }
        });
    }
}
