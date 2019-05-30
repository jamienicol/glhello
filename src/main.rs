use glutin::{ContextBuilder, ControlFlow, Event, EventsLoop, WindowBuilder, WindowEvent};

fn main() {
    let mut event_loop = EventsLoop::new();
    let wb = WindowBuilder::new().with_title("Hello GL!");

    let context = ContextBuilder::new()
        .build_windowed(wb, &event_loop)
        .unwrap();

    let context = unsafe { context.make_current().unwrap() };

    event_loop.run_forever(|event| {
        match event {
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::Resized(logical_size) => {
                    let dpi_factor = context.window().get_hidpi_factor();
                    context.resize(logical_size.to_physical(dpi_factor));
                }
                WindowEvent::Refresh => {
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
