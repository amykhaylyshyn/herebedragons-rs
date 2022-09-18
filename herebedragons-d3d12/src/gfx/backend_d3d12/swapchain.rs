use super::BackendD3D12;

pub struct SwapChain {}

impl SwapChain {
    pub fn new() -> Self {
        Self {}
    }
}

impl crate::gfx::SwapChain<BackendD3D12> for SwapChain {}
