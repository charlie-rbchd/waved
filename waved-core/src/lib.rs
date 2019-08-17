use std::thread_local;

mod renderer;
use renderer::Renderer;

thread_local! {
    #[allow(non_upper_case_globals)]
    static renderer: Renderer<'static> = Renderer::new();
}

#[no_mangle]
pub extern "C" fn render(width: f32, height: f32, scale: f32) {
    renderer.with(|r| r.render(width, height, scale));
}
