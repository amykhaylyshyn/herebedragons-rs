use std::sync::Arc;

use d3d12::DxgiAdapter;

use super::BackendD3D12;

pub struct Adapter {
    lib_d3d12: Arc<d3d12::D3D12Lib>,
    raw: DxgiAdapter,
}

impl Adapter {
    pub fn new(lib_d3d12: Arc<d3d12::D3D12Lib>, raw: DxgiAdapter) -> Self {
        Self { lib_d3d12, raw }
    }

    pub fn raw(&self) -> DxgiAdapter {
        self.raw
    }
}

impl crate::gfx::Adapter<BackendD3D12> for Adapter {}
