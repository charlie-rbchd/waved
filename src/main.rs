// TODO: Split this file
// TODO: Implement wave file parser using nom (in a subproject)
// TODO: Implement hot reloading

use glfw::{Action, Context, Key};
use nfd::Response;
use portaudio as pa;

use std::f64::consts::PI;
use std::thread;

struct AppFonts<'a> {
    regular: nanovg::Font<'a>,
    _bold: nanovg::Font<'a>,
}

#[cfg(target_os = "macos")]
extern "C" fn refresh_callback(window: *mut glfw::ffi::GLFWwindow) {
    // TODO: Make the draw context / fonts global so any render function
    // can access them.
    unsafe {
        let mut logical_width: std::os::raw::c_int = 0;
        let mut logical_height: std::os::raw::c_int = 0;
        glfw::ffi::glfwGetFramebufferSize(window, &mut logical_width, &mut logical_height);

    // let mut physical_width: std::os::raw::c_int = 0;
    // let mut physical_height: std::os::raw::c_int = 0;
    // glfw::ffi::glfwGetWindowSize(window, &mut physical_width, &mut physical_height);

    // let dpi_scale = logical_width as f32 / physical_width as f32;

        gl::Viewport(0, 0, logical_width, logical_height);
        gl::ClearColor(0.2, 0.2, 0.2, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);

        glfw::ffi::glfwSwapBuffers(window);
    }
}

fn main() {
    // Parse the command line
    let matches = clap::App::new("myprog")
        .arg(clap::Arg::with_name("files").multiple(true))
        .get_matches();
    
    if let Some(files) = matches.values_of("files").map(|vals| vals.collect::<Vec<_>>()) {
        println!("Files {:?}", files);
    }

    // Create window
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 2));
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

    let (mut window, events) = glfw.create_window(960, 320, "waved", glfw::WindowMode::Windowed)
        .expect("Failed to create a window.");

    window.set_key_polling(true);
    window.set_drag_and_drop_polling(true);
    if cfg!(target_os = "macos") {
        unsafe {
            // Allow rendering while resizing due to wait_events / poll_events
            // locking the main loop on macOS (see https://github.com/glfw/glfw/issues/1).
            glfw::ffi::glfwSetWindowRefreshCallback(window.window_ptr(), Some(refresh_callback));
        }
    }

    window.make_current();
    gl::load_with(|symbol| window.get_proc_address(symbol));
    glfw.set_swap_interval(glfw::SwapInterval::Sync(1)); // Enable vsync

    let context = nanovg::ContextBuilder::new()
        .antialias()
        .stencil_strokes()
        .build()
        .expect("Failed to create a drawing context.");

    let fonts = AppFonts {
        regular: nanovg::Font::from_file(&context, "Inconsolata-Regular", "resources/Inconsolata-Regular.ttf")
            .expect("Failed to load font 'Inconsolata-Regular.ttf'"),

        _bold: nanovg::Font::from_file(&context, "Inconsolata-Bold", "resources/Inconsolata-Bold.ttf")
            .expect("Failed to load font 'Inconsolata-Bold.ttf'"),
    };

    // TODO: Implement a scene graph
    // TODO: Less frequent redraws (dirty state checking)
    while !window.should_close() {
        render(&mut window, &context, &fonts);

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event);
        }
    }
}

fn render(window: &mut glfw::Window, context: &nanovg::Context, fonts: &AppFonts) {
    let (logical_width, logical_height) = window.get_framebuffer_size();
    let (physical_width, physical_height) = window.get_size();

    let dpi_scale = logical_width as f32 / physical_width as f32;

    unsafe {
        gl::Viewport(0, 0, logical_width, logical_height);
        gl::ClearColor(0.2, 0.2, 0.2, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
    }

    context.frame((physical_width as f32, physical_height as f32), dpi_scale, |frame| {
        nanovg_draw_test(&frame, fonts.regular);
    });

    window.swap_buffers();
}

fn nanovg_draw_test(frame: &nanovg::Frame, font: nanovg::Font) {
    frame.path(
        |path| {
            path.rect((0.0, 0.0), (100.0, 100.0));
            path.fill(nanovg::Color::from_rgb(255, 255, 0), Default::default());
        },
        Default::default()
    );

    frame.text(font, (200.0, 100.0), "Hello, world! Hopefully the text rendering isn't too bad...", nanovg::TextOptions {
        color: nanovg::Color::from_rgb(240, 240, 240),
        align: nanovg::Alignment::new().left().top(),
        size: 16.0,
        ..Default::default()
    });
}

fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent) {
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
            window.set_should_close(true)
        },
        glfw::WindowEvent::Key(Key::O, _, Action::Press, _) => {
            let result = nfd::dialog()
                .filter("wav").open()
                .expect("Failed to open file dialog.");

            match result {
                Response::Okay(file_path) => println!("File path = {:?}", file_path),
                Response::OkayMultiple(_) => panic!("User should only be able to select a single file."),
                Response::Cancel => println!("User canceled"),
            }
        },
        glfw::WindowEvent::Key(Key::S, _, Action::Press, _) => {
            thread::spawn(move || {
                run_portaudio_test().expect("PortAudio Test: failed to run");
            });
        },
        glfw::WindowEvent::FileDrop(files) => {
            println!("Files {:?}", files);
        }
        _ => {}
    }
}

fn run_portaudio_test() -> Result<(), pa::Error> {
    const CHANNELS: i32 = 2;
    const NUM_SECONDS: i32 = 2;
    const SAMPLE_RATE: f64 = 44_100.0;
    const FRAMES_PER_BUFFER: u32 = 64;
    const TABLE_SIZE: usize = 200;

    println!("PortAudio Test: output sine wave. SR = {}, BufSize = {}", SAMPLE_RATE, FRAMES_PER_BUFFER);

    // Initialise sinusoidal wavetable.
    let mut sine = [0.0; TABLE_SIZE];
    for i in 0..TABLE_SIZE {
        sine[i] = (i as f64 / TABLE_SIZE as f64 * PI * 2.0).sin() as f32;
    }
    let mut left_phase = 0;
    let mut right_phase = 0;

    let pa = pa::PortAudio::new()?;

    let mut settings = pa.default_output_stream_settings(CHANNELS, SAMPLE_RATE, FRAMES_PER_BUFFER)?;
    // we won't output out of range samples so don't bother clipping them.
    settings.flags = pa::stream_flags::CLIP_OFF;

    // This routine will be called by the PortAudio engine when audio is needed. It may called at
    // interrupt level on some machines so don't do anything that could mess up the system like
    // dynamic resource allocation or IO.
    let callback = move |pa::OutputStreamCallbackArgs { buffer, frames, .. }| {
        let mut idx = 0;
        for _ in 0..frames {
            buffer[idx]   = sine[left_phase];
            buffer[idx+1] = sine[right_phase];
            left_phase += 1;
            if left_phase >= TABLE_SIZE { left_phase -= TABLE_SIZE; }
            right_phase += 3;
            if right_phase >= TABLE_SIZE { right_phase -= TABLE_SIZE; }
            idx += 2;
        }
        pa::Continue
    };

    let mut stream = pa.open_non_blocking_stream(settings, callback)?;

    stream.start()?;

    println!("Play for {} seconds.", NUM_SECONDS);
    pa.sleep(NUM_SECONDS * 1_000);

    stream.stop()?;
    stream.close()?;

    println!("Test finished.");

    Ok(())
}
