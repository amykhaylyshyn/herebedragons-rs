use d3d12::CommandQueue;

use super::BackendD3D12;

pub struct Queue {
    queue: CommandQueue,
}

impl Queue {
    pub fn new(queue: CommandQueue) -> Self {
        Self { queue }
    }
}

impl Drop for Queue {
    fn drop(&mut self) {
        unsafe { self.queue.destroy() };
    }
}

impl crate::gfx::Queue<BackendD3D12> for Queue {}

pub struct CommandAllocator {
    allocator: d3d12::CommandAllocator,
}

impl CommandAllocator {
    pub fn new(allocator: d3d12::CommandAllocator) -> Self {
        Self { allocator }
    }
}

impl Drop for CommandAllocator {
    fn drop(&mut self) {
        unsafe { self.allocator.destroy() };
    }
}

impl crate::gfx::CommandAllocator<BackendD3D12> for CommandAllocator {}

pub struct CommandList {
    cmd_list: d3d12::CommandList,
}

impl CommandList {
    pub fn new(cmd_list: d3d12::CommandList) -> Self {
        Self { cmd_list }
    }
}

impl Drop for CommandList {
    fn drop(&mut self) {
        unsafe { self.cmd_list.destroy() };
    }
}

impl crate::gfx::CommandList<BackendD3D12> for CommandList {}
