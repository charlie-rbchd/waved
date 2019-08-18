use nanovg::{Alignment, Color, Context, ContextBuilder, Font, TextOptions};
use std::ops::Deref;

use waved_state::State;

#[allow(dead_code)]
pub struct Fonts<'a> {
    regular: Font<'a>,
    bold: Font<'a>,
}

pub struct Renderer<'a> {
    context: Box<Context>,
    fonts: Fonts<'a>,
}

impl<'a> Renderer<'a> {
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
                    .expect("Failed to load font 'Inconsolata-Regular.ttf'"),

                bold: Font::from_file(unsafe { &*context_ptr }, "Inconsolata-Bold", "resources/Inconsolata-Bold.ttf")
                    .expect("Failed to load font 'Inconsolata-Bold.ttf'"),
            }
        };

        Self { context, fonts }
    }

    pub fn render(&self, state: &State, viewport: (f32, f32), scale: f32) {
        self.context.frame(viewport, scale, |frame| {
            frame.path(
                |path| {
                    path.rect((50.0, 50.0), (100.0, 100.0));
                    path.fill(Color::from_rgb(255, 255, 255), Default::default());
                },
                Default::default()
            );

            frame.text(self.fonts.regular, (200.0, 100.0), "Hello, world!", TextOptions {
                color: Color::from_rgb(240, 240, 240),
                align: Alignment::new().left().top(),
                size: 16.0,
                ..Default::default()
            });
        });
    }
}
