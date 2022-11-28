use raw_window_handle::HasRawWindowHandle;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

use crate::gfx::{
    Backend, Device, Format, Instance, NewInstanceOptions, SampleOptions, ScopedResource,
    SwapChainOptions, SwapEffect,
};

use crate::error::Result;

struct BackbufferResources<'a, B: Backend> {
    command_allocator: ScopedResource<'a, B::Device, B::CommandAllocator>,
    fence: ScopedResource<'a, B::Device, B::Fence>,
}

pub fn render_main<B>(width: usize, height: usize, backbuffer_count: usize) -> Result<()>
where
    B: Backend,
{
    let instance = B::Instance::new(NewInstanceOptions {
        enable_debug_layer: true,
    })?;
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
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::Size::Logical(winit::dpi::LogicalSize::new(
            width as f64,
            height as f64,
        )))
        .build(&event_loop)
        .unwrap();
    let window_handle = window.raw_window_handle();

    let swap_chain_options = SwapChainOptions {
        backbuffer_count,
        swap_effect: SwapEffect::Discard,
        sample_options: SampleOptions { sample_count: 8 },
        width: width as _,
        height: height as _,
        format: Format::R8G8B8A8UNorm,
    };
    let swap_chain = instance.create_swap_chain(&window_handle, &queue, &swap_chain_options)?;

    event_loop.run(move |event, _, control_flow| {
        let _ = instance;
        let _ = default_adapter;
        let _ = device;
        let _ = queue;
        let _ = backbuffers;
        let _ = swap_chain;

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
