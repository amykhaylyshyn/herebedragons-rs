use super::BackendD3D12;

pub struct Fence {
    fence: d3d12::Fence,
}

impl Fence {
    pub fn new(fence: d3d12::Fence) -> Self {
        Self { fence }
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        unsafe { self.fence.destroy() };
    }
}

impl crate::gfx::Fence<BackendD3D12> for Fence {}
