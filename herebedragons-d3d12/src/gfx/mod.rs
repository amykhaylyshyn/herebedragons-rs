pub mod backend_d3d12;

use crate::error::Result;

#[derive(Debug, Clone)]
pub struct AdapterDescription {
    pub device_id: u32,
    pub vendor_id: u32,
    pub description: String,
    pub has_hw_acceleration: bool,
}

pub struct AdapterDetails<B: Backend> {
    pub adapter: B::Adapter,
    pub description: AdapterDescription,
}

pub trait Backend: Sized {
    type Factory: Factory<Self>;
    type Adapter: Adapter<Self>;
    type Device: Device<Self>;
    type Queue: Queue<Self>;
    type DescriptorHeap: DescriptorHeap<Self>;
    type CommandAllocator: CommandAllocator<Self>;
    type CommandList: CommandList<Self>;
    type Fence: Fence<Self>;
}

pub trait Factory<B: Backend> {
    fn enumerate_adapters(&self) -> Result<Vec<AdapterDetails<B>>>;
}

pub trait Adapter<B: Backend> {
    fn create_device(&self) -> Result<B::Device>;
}

pub trait Device<B: Backend> {
    fn create_queue(&self) -> Result<B::Queue>;
    fn create_command_allocator(&self) -> Result<B::CommandAllocator>;
    fn create_fence(&self, initial: u64) -> Result<B::Fence>;
}

pub trait Queue<B: Backend> {}

pub trait DescriptorHeap<B: Backend> {}

pub trait CommandAllocator<B: Backend> {}

pub trait CommandList<B: Backend> {}

pub trait Fence<B: Backend> {}
