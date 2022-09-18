pub mod backend_d3d12;

use crate::error::Result;
use derive_more::Deref;
use raw_window_handle::RawWindowHandle;

#[derive(Debug, Clone, Copy)]
pub enum SwapEffect {
    Discard,
    Sequential,
    FlipSequential,
    FlipDiscard,
}

#[derive(Debug, Clone, Copy)]
pub enum Format {
    R8G8B8A8UNorm,
}

#[derive(Debug, Default)]
pub struct NewInstanceOptions {
    pub enable_debug_layer: bool,
}

#[derive(Debug, Default)]
pub struct SampleOptions {
    pub sample_count: usize,
}

#[derive(Debug)]
pub struct SwapChainOptions {
    pub backbuffer_count: usize,
    pub swap_effect: SwapEffect,
    pub sample_options: SampleOptions,
    pub width: u32,
    pub height: u32,
    pub format: Format,
}

pub trait Backend: Sized {
    type Instance: Instance<Self>;
    type Adapter: Adapter<Self>;
    type Device: Device<Self>;
    type Queue: Queue<Self>;
    type DescriptorHeap: DescriptorHeap<Self>;
    type CommandAllocator: CommandAllocator<Self>;
    type CommandList: CommandList<Self>;
    type Fence: Fence<Self>;
    type SwapChain: SwapChain<Self>;
}

pub trait Instance<B: Backend>: Sized {
    fn new(options: NewInstanceOptions) -> Result<Self>;
    fn enumerate_adapters(&self) -> Result<Vec<AdapterDetails<B>>>;
    fn create_device(&self, adapter: &B::Adapter)
        -> Result<ScopedResource<B::Instance, B::Device>>;
    fn create_swap_chain(
        &self,
        raw_window: &RawWindowHandle,
        queue: &B::Queue,
        options: &SwapChainOptions,
    ) -> Result<ScopedResource<B::Instance, B::SwapChain>>;
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

pub trait SwapChain<B: Backend> {}

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
