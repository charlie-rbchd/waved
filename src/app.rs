use glfw::{Action, Context, Glfw, Key, OpenGlProfileHint, SwapInterval, Window, WindowEvent, WindowHint, WindowMode, FAIL_ON_ERRORS};
use libloading::Library;

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::thread_local;

use waved_core::State;

use crate::cli::CommandLineArgs;

#[cfg(target_os = "macos")]
const CORELIB_FILENAME: &'static str = "libwaved_gui.dylib";
#[cfg(target_os = "linux")]
const CORELIB_FILENAME: &'static str = "libwaved_gui.so";
#[cfg(target_os = "windows")]
const CORELIB_FILENAME: &'static str = "waved_gui.dll";

fn dylib_path(lib_filename: &str) -> PathBuf {
    std::env::current_exe().unwrap()
        .parent().unwrap()
        .join(lib_filename)
}

fn dylib_load_path(lib_filename: &str) -> PathBuf {
    let src_path = dylib_path(lib_filename);

    #[cfg(feature = "live-reload")]
    {
        use std::time::{SystemTime, UNIX_EPOCH};

        // Most systems either lock or cache dynamic libraries once they are loaded by an application,
        // make a unique copy of it to allow hot reloading
        let dest_filename = format!("{}-{}.{}",
            src_path.file_stem().unwrap().to_str().unwrap(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            src_path.extension().unwrap().to_str().unwrap());

        let dest_path = src_path.parent().unwrap().join(format!("reloaded/{}", dest_filename));

        std::fs::create_dir_all(dest_path.parent().unwrap()).unwrap();
        std::fs::copy(&src_path, &dest_path).unwrap();

        // Some systems embed an identifier into the dyamic library and uses that for caching,
        // make sure that identifier is updated to reflect the new location of the library
        if cfg!(target_os = "macos") {
            let dest_str = dest_path.to_str().unwrap();
            let status = std::process::Command::new("install_name_tool")
                .args(&["-id", &dest_str, &dest_str])
                .status()
                .expect("Failed to change dylib identifier.");
            assert!(status.success());
        } else if cfg!(target_os = "linux") {
            // TODO: Validate if `patchelf --set-soname` is needed for live reloading
            // to work properly on Linux
        }

        dest_path
    }

    #[cfg(not(feature = "live-reload"))]
    src_path
}

#[cfg(feature = "live-reload")]
fn clean_reloaded_dylib() {
    let reloaded_folder = std::env::current_exe().unwrap()
        .parent().unwrap()
        .join("reloaded");

    if std::fs::remove_dir_all(&reloaded_folder).is_ok() {
        println!("Cleaned up reloaded folder.");
    }
}

extern "C" fn refresh_callback(_window: *mut glfw::ffi::GLFWwindow) {
    app.with(|a| a.render_ui());
}

#[allow(dead_code)]
pub struct App {
    corelib: RefCell<Library>,
    glfw: RefCell<Glfw>,
    window: RefCell<Window>,
    events: Receiver<(f64, WindowEvent)>,
    state: RefCell<State>,
}

thread_local! {
    #[allow(non_upper_case_globals)]
    pub static app: App = App::new();
}

impl App {
    pub fn new() -> Self {
        #[cfg(feature = "live-reload")]
        clean_reloaded_dylib();

        let state = Default::default();
        let corelib = Library::new(dylib_load_path(CORELIB_FILENAME))
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
        unsafe { glfw::ffi::glfwSetWindowRefreshCallback(window.window_ptr(), Some(refresh_callback)); }

        window.make_current();
        gl::load_with(|symbol| window.get_proc_address(symbol));
        glfw.set_swap_interval(SwapInterval::Sync(1)); // Enable vsync

        Self {
            corelib: RefCell::new(corelib),
            glfw: RefCell::new(glfw),
            window: RefCell::new(window),
            events,
            state: RefCell::new(state),
        }
    }

    pub fn render_ui(&self) {
        self.clear();

        let (physical_width, physical_height) = self.window.borrow().get_size();
        let corelib = self.corelib.borrow();
        if let Ok(render) = unsafe { corelib.get::<fn(&State, (f32, f32), f32)>(b"render\0") } {
            render(&self.state.borrow(), (physical_width as f32, physical_height as f32), self.dpi_scale());
        }

        self.window.borrow_mut().swap_buffers();
    }

    pub fn run(&self, args: CommandLineArgs) {
        if !args.files.is_empty() {
            println!("Files {:?}", args.files);
        }

        #[cfg(feature = "live-reload")]
        let mut last_modified = std::fs::metadata(dylib_path(CORELIB_FILENAME)).unwrap()
            .modified().unwrap();

        while !self.window.borrow().should_close() {
            #[cfg(feature = "live-reload")]
            {
                if let Ok(metadata) = std::fs::metadata(dylib_path(CORELIB_FILENAME)) {
                    let modified = metadata.modified().unwrap();
                    if modified > last_modified {
                        drop(self.corelib.borrow_mut());
                        *self.corelib.borrow_mut() = Library::new(dylib_load_path(CORELIB_FILENAME))
                            .expect("Failed to load core library.");

                        last_modified = modified;
                        println!("Reloaded core library!");
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
                    nfd::Response::Okay(filename) => {
                        self.load_file(filename);
                    },
                    nfd::Response::OkayMultiple(_) => panic!("Should only be able to select a single file."),
                    nfd::Response::Cancel => {},
                }
            },
            WindowEvent::Key(Key::S, _, Action::Press, _) => {
                // TODO: Create a single audio thread and move the playback code to another file.
                // thread::spawn(move || {
                //     run_portaudio_test().expect("PortAudio Test: failed to run");
                // });
            },
            WindowEvent::FileDrop(files) => {
                if files.len() > 0 {
                    self.load_file(&files[0]);
                }
            }
            _ => {}
        }
    }

    fn load_file<P: AsRef<Path>>(&self, filename: P) {
        // TODO: To be implemented, read file and dump samples in the state
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
