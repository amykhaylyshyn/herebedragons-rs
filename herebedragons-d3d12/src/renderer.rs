use crate::gfx::{Backend, Device, Instance, ScopedResource};

use crate::error::Result;

struct BackbufferResources<'a, B: Backend> {
    command_allocator: ScopedResource<'a, B::Device, B::CommandAllocator>,
    fence: ScopedResource<'a, B::Device, B::Fence>,
}

pub fn render_loop<B: Backend>(backbuffer_count: usize) -> Result<()> {
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

    Ok(())
}
