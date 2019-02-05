use glfw::{Action, Context, Key};
use nfd::Response;

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    let (mut window, events) = glfw.create_window(960, 320, "waved", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.set_key_polling(true);
    window.make_current();

    // TODO: Create a cairo context from the native context
    // and draw something on it!

    // let (width, height) = window.get_size();
    // let surface = cairo::QuartzSurface::create_for_cg_context(width, height);

    // let cr = cairo::Context::new(&surface);
    // cr.scale(120.0, 120.0);
    // cr.set_line_width(0.1);
    // cr.set_source_rgb(0.0, 0.0, 0.0);
    // cr.rectangle(0.25, 0.25, 0.5, 0.5);
    // cr.stroke();

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event);
        }
    }
}

fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent) {
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
            window.set_should_close(true)
        },
        glfw::WindowEvent::Key(Key::O, _, Action::Press, _) => {
            let result = nfd::open_file_dialog(None, None).unwrap_or_else(|e| {
                panic!(e);
            });

            match result {
                Response::Okay(file_path) => println!("File path = {:?}", file_path),
                Response::OkayMultiple(files) => println!("Files {:?}", files),
                Response::Cancel => println!("User canceled"),
            }
        },
        _ => {}
    }
}
