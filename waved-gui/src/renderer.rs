#![allow(dead_code)]

use nanovg::{Color, Context, ContextBuilder, Font, StrokeOptions};

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

enum DisplayError {}

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
            let width = viewport.0;
            let height = viewport.1;

            if let Some(file) = &state.current_file {
                let num_samples = file.samples.len();

                let mut current_x = 0;
                let mut current_avg = 0.0;
                let mut num_samples_acc = 0;

                // TODO: Multi-channel rendering
                for (i, s) in file.samples.iter().enumerate() {
                    let x = (i as f32 / num_samples as f32 * width).round() as i32;
                    if current_x == x {
                        current_avg += s.abs();
                        num_samples_acc += 1;
                    } else {
                        current_avg /= num_samples_acc as f32;

                        let line_height = height * current_avg;
                        let line_start = (height - line_height) * 0.5;
                        let line_end = line_start + line_height;

                        frame.path(
                            |path| {
                                path.move_to((x as f32, line_start));
                                path.line_to((x as f32, line_end));
                                path.stroke(
                                    Color::from_rgba(255, 255, 255, 255),
                                    StrokeOptions {
                                        width: 1.0,
                                        ..Default::default()
                                    }
                                );
                            },
                            Default::default(),
                        );
                        current_x = x;
                        current_avg = 0.0;
                        num_samples_acc = 0;
                    }
                }
            }
        });
    }
}
