use std::sync::Arc;

use d3d12::DxgiAdapter;
use winapi::shared::dxgi;

use crate::{error::RenderDeviceError, hresult::IntoResult};

use super::{Backend, Device};

pub struct Adapter {
    lib_d3d12: Arc<d3d12::D3D12Lib>,
    adapter: DxgiAdapter,
}

impl Adapter {
    pub fn new(lib_d3d12: Arc<d3d12::D3D12Lib>, adapter: DxgiAdapter) -> Self {
        Self { lib_d3d12, adapter }
    }
}

impl crate::gfx::Adapter<Backend> for Adapter {
    fn create_device(&self) -> crate::error::Result<Device> {
        let adapter = unsafe { self.adapter.cast::<dxgi::IDXGIAdapter1>().into_result()? };
        let device = self
            .lib_d3d12
            .create_device(adapter, d3d12::FeatureLevel::L11_0)
            .map_err(|err| {
                log::error!("Create D3D12 error {}", err);
                RenderDeviceError::LoadLibraryError
            })?
            .into_result()?;
        Ok(Device::new(device))
    }
}
