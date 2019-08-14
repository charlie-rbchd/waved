#[no_mangle]
pub fn render(frame: &nanovg::Frame, font: &nanovg::Font) {
    frame.path(
        |path| {
            path.rect((50.0, 50.0), (100.0, 100.0));
            path.fill(nanovg::Color::from_rgb(255, 255, 255), Default::default());
        },
        Default::default()

    );

    frame.text(*font, (200.0, 100.0), "Hello, world!", nanovg::TextOptions {
        color: nanovg::Color::from_rgb(240, 240, 240),
        align: nanovg::Alignment::new().left().top(),
        size: 16.0,
        ..Default::default()
    });
}
