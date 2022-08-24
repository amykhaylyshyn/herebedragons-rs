use std::sync::Arc;

use d3d12::DxgiAdapter;

use super::BackendD3D12;

pub struct Adapter {
    raw: DxgiAdapter,
}

impl Adapter {
    pub fn new(raw: DxgiAdapter) -> Self {
        Self { raw }
    }

    pub fn raw(&self) -> DxgiAdapter {
        self.raw
    }
}

impl crate::gfx::Adapter<BackendD3D12> for Adapter {}
