use gleam::gl;
use glutin::{Api, ContextBuilder, ControlFlow, Event, EventsLoop, GlRequest, WindowBuilder, WindowEvent};

fn main() {
    let mut event_loop = EventsLoop::new();
    let wb = WindowBuilder::new().with_title("Hello GL!");

    let context = ContextBuilder::new()
        .with_gl(GlRequest::Specific(Api::OpenGlEs, (3, 0)))
        .build_windowed(wb, &event_loop)
        .unwrap();

    let context = unsafe { context.make_current().unwrap() };

    let gl = gl::ErrorReactingGl::wrap(
        unsafe { gl::GlesFns::load_with(|s| context.get_proc_address(s) as *const _) },
        move |_gl, function_name, err| {
            println!("Error 0x{:x} in {}", err, function_name);
        },
    );
    gl.clear_color(0.0, 0.0, 0.0, 1.0);

    println!("Vendor: {}", gl.get_string(gl::VENDOR));
    println!("Renderer: {}", gl.get_string(gl::RENDERER));
    println!("Version: {}", gl.get_string(gl::VERSION));

    event_loop.run_forever(|event| {
        match event {
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::Resized(logical_size) => {
                    let dpi_factor = context.window().get_hidpi_factor();
                    context.resize(logical_size.to_physical(dpi_factor));
                }
                WindowEvent::Refresh => {
                    gl.clear(gl::COLOR_BUFFER_BIT);
                    context.swap_buffers().unwrap();
                }
                WindowEvent::CloseRequested => return ControlFlow::Break,
                _ => (),
            },
            _ => (),
        }

        return ControlFlow::Continue;
    });
}
