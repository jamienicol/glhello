use gl;
use glutin::{ContextBuilder, ControlFlow, Event, EventsLoop, WindowBuilder, WindowEvent};

fn main() {
    let mut event_loop = EventsLoop::new();
    let wb = WindowBuilder::new().with_title("Hello GL!");

    let context = ContextBuilder::new()
        .build_windowed(wb, &event_loop)
        .unwrap();

    let context = unsafe { context.make_current().unwrap() };

    gl::load_with(|s| context.get_proc_address(s) as *const _);
    unsafe { gl::ClearColor(0.0, 0.0, 0.0, 1.0); }

    event_loop.run_forever(|event| {
        match event {
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::Resized(logical_size) => {
                    let dpi_factor = context.window().get_hidpi_factor();
                    context.resize(logical_size.to_physical(dpi_factor));
                }
                WindowEvent::Refresh => {
                    unsafe { gl::Clear(gl::COLOR_BUFFER_BIT); }
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
