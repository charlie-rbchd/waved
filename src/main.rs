use glfw::{Action, Key, Window, WindowEvent, WindowMode, WindowHint, ClientApiHint};
use nfd::Response;

use cairo::{QuartzSurface, Context};

#[cfg(target_os = "macos")]
use foreign_types::{ForeignType, ForeignTypeRef};

#[cfg(target_os = "macos")]
use objc::{msg_send, class, sel, sel_impl};

#[cfg(target_os = "macos")]
use core_graphics::{
    color_space::{CGColorSpace, kCGColorSpaceGenericRGB},
    context::{CGContext, CGContextRef},
    base::kCGImageAlphaPremultipliedLast,
};

#[cfg(target_os = "macos")]
use cocoa::base::id;

use std::ptr;

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    // glfw.window_hint(WindowHint::ClientApi(ClientApiHint::NoApi));

    let (mut window, events) = glfw.create_window(960, 320, "waved", WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.set_key_polling(true);
    // window.make_current();

    let (width, height) = window.get_size();

    // let colorspace = CGColorSpace::create_with_name(unsafe { kCGColorSpaceGenericRGB }).unwrap();
    // let cg_context = CGContext::create_bitmap_context(
    //     None, width as usize, height as usize, 8, 0, &colorspace, kCGImageAlphaPremultipliedLast);

    let cocoa_ctx: id = unsafe { msg_send![class!(NSGraphicsContext), currentContext] };
    let coregraphics_ctx: CGContextRef = unsafe { msg_send![cocoa_ctx, graphicsPort] };

    coregraphics_ctx.translate(0.0, height as f64);
    coregraphics_ctx.scale(1.0, -1.0);

    let surface = QuartzSurface::create_for_cg_context(
        coregraphics_ctx.as_ptr() as *mut _, width as u32, height as u32).unwrap();

    let ctx = Context::new(&surface);
    // ctx.set_line_width(0.1);
    ctx.set_source_rgb(0.0, 1.0, 0.0);
    ctx.rectangle(0.25, 0.25, 0.5, 0.5);
    // ctx.stroke();
    ctx.fill();

    coregraphics_ctx.flush();

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event);
        }
    }
}

fn handle_window_event(window: &mut Window, event: glfw::WindowEvent) {
    match event {
        WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
            window.set_should_close(true)
        },
        WindowEvent::Key(Key::O, _, Action::Press, _) => {
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
