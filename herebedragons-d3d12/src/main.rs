mod error;
pub mod gfx;
mod hresult;

use gfx::Instance;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let instance = gfx::backend_d3d12::Instance::new()?;
    instance.enumerate_adapters();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        let _ = &instance;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => control_flow.set_exit(),
            _ => (),
        }
    });
}
