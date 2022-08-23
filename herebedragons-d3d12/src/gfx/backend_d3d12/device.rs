use d3d12::{CmdListType, CommandQueueFlags};

use crate::error::Result;
use crate::hresult::IntoResult;

use super::{Backend, CommandAllocator, Fence, Queue};

pub struct Device {
    device: d3d12::Device,
}

impl Device {
    pub fn new(device: d3d12::Device) -> Self {
        Self { device }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe { self.device.destroy() };
    }
}

impl crate::gfx::Device<Backend> for Device {
    fn create_queue(&self) -> Result<Queue> {
        let queue = self
            .device
            .create_command_queue(
                CmdListType::Direct,
                d3d12::Priority::Normal,
                CommandQueueFlags::empty(),
                Default::default(),
            )
            .into_result()?;
        Ok(Queue::new(queue))
    }

    fn create_command_allocator(&self) -> Result<CommandAllocator> {
        let allocator = self
            .device
            .create_command_allocator(CmdListType::Direct)
            .into_result()?;
        Ok(CommandAllocator::new(allocator))
    }

    fn create_fence(&self, initial: u64) -> Result<Fence> {
        let fence = self.device.create_fence(initial).into_result()?;
        Ok(Fence::new(fence))
    }
}
