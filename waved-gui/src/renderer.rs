#![allow(dead_code)]

use nanovg::{Context, ContextBuilder, Font};

use std::ops::Deref;

use waved_core::State;

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

    pub fn render(&self, _state: &State, viewport: (f32, f32), scale: f32) {
        self.context.frame(viewport, scale, |_frame| {
            // TODO: Render waveform from state
        });
    }
}
