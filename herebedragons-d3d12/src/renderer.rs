use crate::gfx::{Adapter, Backend, Device, Instance};

use crate::error::Result;

struct BackbufferResources<B: Backend> {
    command_allocator: B::CommandAllocator,
    fence: B::Fence,
}

pub struct Renderer<B: Backend> {
    backbuffers: Vec<BackbufferResources<B>>,
    queue: B::Queue,
    device: B::Device,
    adapter: B::Adapter,
    instance: B::Instance,
}

impl<B: Backend> Renderer<B> {
    pub fn new(instance: B::Instance, backbuffer_count: usize) -> Result<Self> {
        let adapters = instance.enumerate_adapters()?;
        let selected_adapter = adapters.into_iter().next().expect("no graphics adapter");
        let device = selected_adapter.adapter.create_device()?;
        let queue = device.create_queue()?;
        let backbuffers = (0..backbuffer_count)
            .map(|_| {
                Ok(BackbufferResources {
                    command_allocator: device.create_command_allocator()?,
                    fence: device.create_fence(0)?,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        log::info!("selected GPU: {:?}", selected_adapter.description);

        Ok(Self {
            instance,
            adapter: selected_adapter.adapter,
            device,
            queue,
            backbuffers,
        })
    }
}
