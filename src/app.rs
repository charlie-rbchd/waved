use glfw::{Action, Context, Glfw, Key, OpenGlProfileHint, SwapInterval, Window, WindowEvent, WindowHint, WindowMode, FAIL_ON_ERRORS};
use libloading::Library;
use cpal::{self, StreamData, UnknownTypeOutputBuffer};
use cpal::traits::{HostTrait, DeviceTrait, EventLoopTrait};
use ringbuf::{self, RingBuffer};

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::thread_local;
use std::thread;

use waved_core::state::{AudioFile, State};
use waved_core::log::Logger;
use waved_sndfile::samples_from_file;

use crate::cli::CommandLineArgs;

#[cfg(target_os = "macos")]
const GUILIB_FILENAME: &'static str = "libwaved_gui.dylib";
#[cfg(target_os = "linux")]
const GUILIB_FILENAME: &'static str = "libwaved_gui.so";
#[cfg(target_os = "windows")]
const GUILIB_FILENAME: &'static str = "waved_gui.dll";

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
    app.with(|a| a.render_gui());
}

#[allow(dead_code)]
struct AudioOutputDevice {
    stream: ringbuf::Producer<f32>,
    channels: u16,
    sample_rate: u32,
}

impl AudioOutputDevice {
    fn new(buffer_size: usize) -> AudioOutputDevice {
        let ring = RingBuffer::new(buffer_size);
        let (producer, _consumer) = ring.split();

        let host = cpal::default_host();
        let device = host.default_output_device().expect("No output device available.");

        let mut supported_formats_range = device.supported_output_formats()
            .expect("Error while querying formats.");
        let format = supported_formats_range.next()
            .expect("No supported format?!")
            .with_max_sample_rate();
        let channels = format.channels;
        let sample_rate = format.sample_rate.0;

        thread::spawn(move || {
            let event_loop = host.event_loop();
            let stream_id = event_loop.build_output_stream(&device, &format).unwrap();
            event_loop.play_stream(stream_id).expect("Failed to play_stream.");

            event_loop.run(move |stream_id, stream_result| {
                let stream_data = match stream_result {
                    Ok(data) => data,
                    Err(err) => {
                        eprintln!("An error occurred on stream {:?}: {}", stream_id, err);
                        return;
                    },
                };
            
                // TODO: Output the samples in the ringbuffer
                // TODO: Can we only support f32 output buffers?
                match stream_data {
                    StreamData::Output { buffer: UnknownTypeOutputBuffer::U16(mut buffer) } => {
                        for elem in buffer.iter_mut() {
                            *elem = u16::max_value() / 2;
                        }
                    },
                    StreamData::Output { buffer: UnknownTypeOutputBuffer::I16(mut buffer) } => {
                        for elem in buffer.iter_mut() {
                            *elem = 0;
                        }
                    },
                    StreamData::Output { buffer: UnknownTypeOutputBuffer::F32(mut buffer) } => {
                        for elem in buffer.iter_mut() {
                            *elem = 0.0;
                        }
                    },
                    _ => (),
                }
            });
        });

        AudioOutputDevice { stream: producer, channels, sample_rate }
    }
}

#[allow(dead_code)]
pub struct App {
    gui: RefCell<Library>,
    glfw: RefCell<Glfw>,
    window: RefCell<Window>,
    events: Receiver<(f64, WindowEvent)>,
    state: RefCell<State>,
    logger: RefCell<Logger>,
    audio_out: AudioOutputDevice,
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
        let gui = Library::new(dylib_load_path(GUILIB_FILENAME))
            .expect("Failed to load core library.");

        let logger = Logger::new();

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

        const BUFFER_SIZE: usize = 512;
        let audio_out = AudioOutputDevice::new(BUFFER_SIZE);

        Self {
            gui: RefCell::new(gui),
            glfw: RefCell::new(glfw),
            window: RefCell::new(window),
            events,
            state: RefCell::new(state),
            logger: RefCell::new(logger),
            audio_out,
        }
    }

    pub fn render_gui(&self) {
        self.clear();

        let (physical_width, physical_height) = self.window.borrow().get_size();
        let gui = self.gui.borrow();
        if let Ok(render) = unsafe { gui.get::<fn(&State, (f32, f32), f32)>(b"render\0") } {
            render(&self.state.borrow(), (physical_width as f32, physical_height as f32), self.dpi_scale());
        }

        self.window.borrow_mut().swap_buffers();
    }

    pub fn run(&self, args: CommandLineArgs) {
        if !args.files.is_empty() {
            println!("Files {:?}", args.files);
        }

        #[cfg(feature = "live-reload")]
        let mut last_modified = std::fs::metadata(dylib_path(GUILIB_FILENAME)).unwrap()
            .modified().unwrap();

        while !self.window.borrow().should_close() {
            #[cfg(feature = "live-reload")]
            {
                if let Ok(metadata) = std::fs::metadata(dylib_path(GUILIB_FILENAME)) {
                    let modified = metadata.modified().unwrap();
                    if modified > last_modified {
                        drop(self.gui.borrow_mut());
                        *self.gui.borrow_mut() = Library::new(dylib_load_path(GUILIB_FILENAME))
                            .expect("Failed to load core library.");

                        last_modified = modified;
                        println!("Reloaded core library!");
                    }
                }
            }

            self.render_gui();

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
            WindowEvent::Key(Key::Space, _, Action::Press, _) => {
            },
            WindowEvent::FileDrop(files) => {
                if files.len() > 0 {
                    self.load_file(&files[0]);
                }
            }
            _ => {}
        }
    }

    fn load_file<P: AsRef<Path> + Into<PathBuf>>(&self, filename: P) {
        match samples_from_file(&filename) {
            Ok((samples, channels, sample_rate)) => {
                self.state.borrow_mut().current_file = Some(AudioFile {
                    filename: filename.into(),
                    samples,
                    channels,
                    sample_rate
                })
            },
            Err(err) => { dbg!(err); },
        }
    }
}
