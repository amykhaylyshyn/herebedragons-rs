use d3d12::DxgiAdapter;

use super::Backend;

pub struct Adapter {
    adapter: DxgiAdapter,
}

impl Adapter {
    pub fn new(adapter: DxgiAdapter) -> Self {
        Self { adapter }
    }
}

impl crate::gfx::Adapter<Backend> for Adapter {}
