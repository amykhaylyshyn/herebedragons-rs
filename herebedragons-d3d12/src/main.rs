mod error;
mod gfx;
mod hresult;
mod renderer;

use dotenv::dotenv;
use gfx::{Adapter, Factory};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let instance = gfx::backend_d3d12::Factory::new()?;
    let adapters = instance.enumerate_adapters()?;
    let selected_adapter = adapters.into_iter().next().expect("no graphics adapter");
    let device = selected_adapter.adapter.create_device()?;

    log::info!("selected GPU: {:?}", selected_adapter.description);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        let _ = &instance;
        let _ = &device;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => control_flow.set_exit(),
            _ => (),
        }
    });
}
