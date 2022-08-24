pub mod backend_d3d12;

use crate::error::Result;
use derive_more::Deref;

pub trait Backend: Sized {
    type Instance: Instance<Self>;
    type Adapter: Adapter<Self>;
    type Device: Device<Self>;
    type Queue: Queue<Self>;
    type DescriptorHeap: DescriptorHeap<Self>;
    type CommandAllocator: CommandAllocator<Self>;
    type CommandList: CommandList<Self>;
    type Fence: Fence<Self>;
}

pub trait Instance<B: Backend> {
    fn enumerate_adapters(&self) -> Result<Vec<AdapterDetails<B>>>;
    fn create_device(&self, adapter: &B::Adapter)
        -> Result<ScopedResource<B::Instance, B::Device>>;
}

pub trait Adapter<B: Backend> {}

pub trait Device<B: Backend> {
    fn create_queue(&self) -> Result<ScopedResource<B::Device, B::Queue>>;
    fn create_command_allocator(&self) -> Result<ScopedResource<B::Device, B::CommandAllocator>>;
    fn create_fence(&self, initial: u64) -> Result<ScopedResource<B::Device, B::Fence>>;
}

pub trait Queue<B: Backend> {}

pub trait DescriptorHeap<B: Backend> {}

pub trait CommandAllocator<B: Backend> {}

pub trait CommandList<B: Backend> {}

pub trait Fence<B: Backend> {}

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

#[derive(Debug, Deref)]
pub struct ScopedResource<'a, P, T> {
    parent: &'a P,
    #[deref]
    resource: T,
}

impl<'a, P, T> ScopedResource<'a, P, T> {
    pub fn new(parent: &'a P, resource: T) -> Self {
        Self { parent, resource }
    }
}
