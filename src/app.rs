use glfw::{ Action, Context, Glfw, Key, OpenGlProfileHint, SwapInterval, Window, WindowEvent, WindowHint, WindowMode, FAIL_ON_ERRORS };

use libloading::Library;

use std::thread_local;
use std::cell::RefCell;
use std::ops::Deref;
use std::sync::mpsc::Receiver;

use crate::cli::CommandLineArgs;

#[cfg(all(target_os = "macos", debug_assertions))]
const CORELIB_PATH: &str = "waved-core/target/debug/libwaved_core.dylib";
#[cfg(all(target_os = "macos", not(debug_assertions)))]
const CORELIB_PATH: &str = "waved-core/target/release/libwaved_core.dylib";
#[cfg(all(target_os = "windows", debug_assertions))]
const CORELIB_PATH: &str = "waved-core/target/debug/libwaved_core.dll";
#[cfg(all(target_os = "windows", not(debug_assertions)))]
const CORELIB_PATH: &str = "waved-core/target/release/libwaved_core.dll";
#[cfg(all(target_os = "linux", debug_assertions))]
const CORELIB_PATH: &str = "waved-core/target/debug/libwaved_core.so";
#[cfg(all(target_os = "linux", not(debug_assertions)))]
const CORELIB_PATH: &str = "waved-core/target/release/libwaved_core.so";

#[cfg(target_os = "macos")]
extern "C" fn refresh_callback(_window: *mut glfw::ffi::GLFWwindow) {
    app.with(|a| a.render_ui());
}

#[allow(dead_code)]
struct Fonts<'a> {
    regular: nanovg::Font<'a>,
    bold: nanovg::Font<'a>,
}

pub struct App<'a> {
    corelib: RefCell<Library>,
    glfw: RefCell<Glfw>,
    window: RefCell<Window>,
    events: Receiver<(f64, WindowEvent)>,
    context: Box<nanovg::Context>,
    fonts: Fonts<'a>,
}

thread_local! {
    pub static app: App<'static> = App::new();
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        let corelib = Library::new(CORELIB_PATH)
            .expect("Failed to load core library.");

        let mut glfw = glfw::init(FAIL_ON_ERRORS).unwrap();

        glfw.window_hint(WindowHint::ContextVersion(3, 2));
        glfw.window_hint(WindowHint::OpenGlForwardCompat(true));
        glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));

        let (mut window, events) = glfw.create_window(960, 320, "waved", WindowMode::Windowed)
            .expect("Failed to create a window.");

        window.set_key_polling(true);
        window.set_drag_and_drop_polling(true);

        // Allow rendering while resizing due to wait_events / poll_events
        // locking the main loop on macOS (see https://github.com/glfw/glfw/issues/1).
        if cfg!(target_os = "macos") { unsafe {
            glfw::ffi::glfwSetWindowRefreshCallback(window.window_ptr(), Some(refresh_callback));
        } }

        window.make_current();
        gl::load_with(|symbol| window.get_proc_address(symbol));
        glfw.set_swap_interval(SwapInterval::Sync(1)); // Enable vsync

        // Has to be heap-allocated since we take it's address when creating fonts.
        let context = Box::new(nanovg::ContextBuilder::new()
            .antialias()
            .stencil_strokes()
            .build()
            .expect("Failed to create a drawing context."));

        // Perform some unsafe pointer gymnastics to ignore lifetime constraints,
        // making it possible to store context and fonts in the same struct even though
        // context would normally have to outlive fonts since it is borrowed in
        // the call to nanovg::Font::from_file.
        let fonts = {
            let context_ptr = context.deref() as *const _;
            Fonts {
                regular: nanovg::Font::from_file(unsafe { &*context_ptr }, "Inconsolata-Regular", "resources/Inconsolata-Regular.ttf")
                    .expect("Failed to load font 'Inconsolata-Regular.ttf'"),

                bold: nanovg::Font::from_file(unsafe { &*context_ptr }, "Inconsolata-Bold", "resources/Inconsolata-Bold.ttf")
                    .expect("Failed to load font 'Inconsolata-Bold.ttf'"),
            }
        };
        
        Self {
            corelib: RefCell::new(corelib),
            glfw: RefCell::new(glfw),
            window: RefCell::new(window),
            events,
            context,
            fonts,
        }
    }

    pub fn render_ui(&self) {
        self.clear();

        // TODO: Implement a scene graph
        // TODO: Less frequent redraws (dirty state checking)
        let (physical_width, physical_height) = self.window.borrow().get_size();
        self.context.frame((physical_width as f32, physical_height as f32), self.dpi_scale(), |frame| {
            frame.path(
                |path| {
                    path.rect((0.0, 0.0), (100.0, 100.0));
                    path.fill(nanovg::Color::from_rgb(255, 255, 0), Default::default());
                },
                Default::default()
            );

            frame.text(self.fonts.regular, (200.0, 100.0), self.get_message(), nanovg::TextOptions {
                color: nanovg::Color::from_rgb(240, 240, 240),
                align: nanovg::Alignment::new().left().top(),
                size: 16.0,
                ..Default::default()
            });
        });

        self.window.borrow_mut().swap_buffers();
    }

    pub fn run(&self, args: CommandLineArgs) {
        if !args.files.is_empty() {
            println!("Files {:?}", args.files);
        }

        #[cfg(debug_assertions)]
        let mut last_modified = std::fs::metadata(CORELIB_PATH).unwrap()
            .modified().unwrap();

        while !self.window.borrow().should_close() {
            // Enable live reloading of core lib in debug
            if cfg!(debug_assertions) {
                if let Ok(Ok(modified)) = std::fs::metadata(CORELIB_PATH).map(|m| m.modified()) {
                    if modified > last_modified {
                        drop(self.corelib.borrow_mut());
                        self.corelib.replace(Library::new(CORELIB_PATH)
                            .expect("Failed to load core library."));

                        last_modified = modified;
                        println!("Reloaded core library!")
                    }
                }
            }

            self.render_ui();

            self.glfw.borrow_mut().poll_events();
            for (_, event) in glfw::flush_messages(&self.events) {
                self.process_event(event);
            }
        }
    }

    fn get_message(&self) -> &'static str { unsafe {
        let l = self.corelib.borrow();
        let f = l.get::<fn() -> &'static str>(b"get_message\0").unwrap();
        f()
    } }

    fn dpi_scale(&self) -> f32 {
        let (logical_width, _) = self.window.borrow().get_framebuffer_size();
        let (physical_width, _) = self.window.borrow().get_size();

        logical_width as f32 / physical_width as f32
    }

    fn clear(&self) {
        let (logical_width, logical_height) = self.window.borrow().get_framebuffer_size();
        unsafe {
            gl::Viewport(0, 0, logical_width, logical_height);
            gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
        }
    }

    fn process_event(&self, event: WindowEvent) {
        match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                self.window.borrow_mut().set_should_close(true)
            },
            WindowEvent::Key(Key::O, _, Action::Press, _) => {
                let result = nfd::dialog()
                    .filter("wav").open()
                    .expect("Failed to open file dialog.");

                match result {
                    nfd::Response::Okay(file_path) => println!("File path = {:?}", file_path),
                    nfd::Response::OkayMultiple(_) => panic!("User should only be able to select a single file."),
                    nfd::Response::Cancel => println!("User canceled"),
                }
            },
            WindowEvent::Key(Key::S, _, Action::Press, _) => {
                // TODO: Create a single audio thread and move the playback code to another file.
                // thread::spawn(move || {
                //     run_portaudio_test().expect("PortAudio Test: failed to run");
                // });
            },
            WindowEvent::FileDrop(files) => {
                println!("Files {:?}", files);
            }
            _ => {}
        }
    }
}

// fn run_portaudio_test() -> Result<(), pa::Error> {
//     const CHANNELS: i32 = 2;
//     const NUM_SECONDS: i32 = 2;
//     const SAMPLE_RATE: f64 = 44_100.0;
//     const FRAMES_PER_BUFFER: u32 = 64;
//     const TABLE_SIZE: usize = 200;

//     println!("PortAudio Test: output sine wave. SR = {}, BufSize = {}", SAMPLE_RATE, FRAMES_PER_BUFFER);

//     // Initialise sinusoidal wavetable.
//     let mut sine = [0.0; TABLE_SIZE];
//     for i in 0..TABLE_SIZE {
//         sine[i] = (i as f64 / TABLE_SIZE as f64 * PI * 2.0).sin() as f32;
//     }
//     let mut left_phase = 0;
//     let mut right_phase = 0;

//     let pa = pa::PortAudio::new()?;

//     let mut settings = pa.default_output_stream_settings(CHANNELS, SAMPLE_RATE, FRAMES_PER_BUFFER)?;
//     // we won't output out of range samples so don't bother clipping them.
//     settings.flags = pa::stream_flags::CLIP_OFF;

//     // This routine will be called by the PortAudio engine when audio is needed. It may called at
//     // interrupt level on some machines so don't do anything that could mess up the system like
//     // dynamic resource allocation or IO.
//     let callback = move |pa::OutputStreamCallbackArgs { buffer, frames, .. }| {
//         let mut idx = 0;
//         for _ in 0..frames {
//             buffer[idx]   = sine[left_phase];
//             buffer[idx+1] = sine[right_phase];
//             left_phase += 1;
//             if left_phase >= TABLE_SIZE { left_phase -= TABLE_SIZE; }
//             right_phase += 3;
//             if right_phase >= TABLE_SIZE { right_phase -= TABLE_SIZE; }
//             idx += 2;
//         }
//         pa::Continue
//     };

//     let mut stream = pa.open_non_blocking_stream(settings, callback)?;

//     stream.start()?;

//     println!("Play for {} seconds.", NUM_SECONDS);
//     pa.sleep(NUM_SECONDS * 1_000);

//     stream.stop()?;
//     stream.close()?;

//     println!("Test finished.");

//     Ok(())
// }
