mod adapter;
mod device;
mod fence;
mod instance;
mod memory;
mod queue;

pub use adapter::*;
pub use device::*;
pub use fence::*;
pub use instance::*;
pub use memory::*;
pub use queue::*;

#[derive(Clone)]
pub struct Backend;

impl crate::gfx::Backend for Backend {
    type Instance = Instance;
    type Adapter = Adapter;
    type Device = Device;
    type Queue = Queue;
    type DescriptorHeap = DescriptorHeap;
    type CommandAllocator = CommandAllocator;
    type CommandList = CommandList;
    type Fence = Fence;
}
