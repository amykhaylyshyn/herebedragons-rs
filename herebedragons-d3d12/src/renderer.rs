use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

use crate::gfx::{Backend, Device, Instance, ScopedResource};

use crate::error::Result;

struct BackbufferResources<'a, B: Backend> {
    command_allocator: ScopedResource<'a, B::Device, B::CommandAllocator>,
    fence: ScopedResource<'a, B::Device, B::Fence>,
}

pub fn render_main<B>(backbuffer_count: usize) -> Result<()>
where
    B: Backend,
{
    let instance = B::Instance::new()?;
    let adapters = instance.enumerate_adapters()?;
    let default_adapter = adapters.into_iter().next().expect("no graphics adapter");
    let device = instance.create_device(&default_adapter.adapter)?;
    let queue = device.create_queue()?;
    let backbuffers = (0..backbuffer_count)
        .map(|_| {
            Ok(BackbufferResources {
                command_allocator: device.create_command_allocator()?,
                fence: device.create_fence(0)?,
            })
        })
        .collect::<Result<Vec<BackbufferResources<B>>>>()?;

    log::info!("selected GPU: {:?}", default_adapter.description);

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    event_loop.run(move |event, _, control_flow| {
        let _ = instance;
        let _ = default_adapter;
        let _ = device;
        let _ = queue;
        let _ = backbuffers;

        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => control_flow.set_exit(),
            _ => (),
        }
    });
}
