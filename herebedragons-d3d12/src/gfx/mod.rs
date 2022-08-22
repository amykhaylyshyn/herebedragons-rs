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
    type Instance: Instance<Self>;
    type Adapter: Adapter<Self>;
    type Device: Device<Self>;
}

pub trait Instance<B: Backend> {
    fn enumerate_adapters(&self) -> Result<Vec<AdapterDetails<B>>>;
}

pub trait Adapter<B: Backend> {
    fn create_device(&self) -> Result<B::Device>;
}

pub trait Device<B: Backend> {}
