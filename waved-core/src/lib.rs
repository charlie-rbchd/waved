use std::thread_local;

use waved_state::State;
mod renderer;
use renderer::Renderer;

thread_local! {
    #[allow(non_upper_case_globals)]
    static renderer: Renderer<'static> = Renderer::new();
}

#[no_mangle]
pub fn render(state: &State, viewport: (f32, f32), scale: f32) {
    renderer.with(|r| r.render(state, viewport, scale));
}
