use std::sync::Arc;

use d3d12::{DxgiAdapter, DxgiFactory};
use winapi::{
    shared::{
        dxgi::{self, DXGI_ADAPTER_DESC1, DXGI_ADAPTER_FLAG_SOFTWARE},
        dxgi1_6, winerror,
    },
    Interface,
};
use windows::core::{HRESULT, HSTRING};

use crate::{
    error::{Error, RenderDeviceError, Result},
    gfx::{AdapterDescription, AdapterDetails},
    hresult::IntoResult,
};

use super::{Adapter, Backend};

pub struct Instance {
    lib_d3d12: Arc<d3d12::D3D12Lib>,
    lib_dxgi: d3d12::DxgiLib,
    factory: d3d12::DxgiFactory,
}

impl Instance {
    pub fn new() -> Result<Self> {
        let lib_d3d12 = d3d12::D3D12Lib::new().map_err(|err| {
            log::error!("load d3d12 library error: {}", err);
            RenderDeviceError::LoadLibraryError
        })?;
        let lib_dxgi = d3d12::DxgiLib::new().map_err(|err| {
            log::error!("load dxgi library error: {}", err);
            RenderDeviceError::LoadLibraryError
        })?;
        let factory = unsafe {
            DxgiFactory::from_factory1(
                lib_dxgi
                    .create_factory1()
                    .map_err(|err| {
                        log::error!("load dxgi library error: {}", err);
                        RenderDeviceError::LoadLibraryError
                    })?
                    .into_result()?,
            )
        };

        Ok(Self {
            lib_d3d12: Arc::new(lib_d3d12),
            lib_dxgi,
            factory,
        })
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            self.factory.destroy();
        }
    }
}

impl crate::gfx::Instance<Backend> for Instance {
    fn enumerate_adapters(&self) -> Result<Vec<AdapterDetails<Backend>>> {
        let factory6 = unsafe { self.factory.cast::<dxgi1_6::IDXGIFactory6>().into_result() }?;
        (0..)
            .map(|adapter_index| -> Result<_> {
                let mut adapter = d3d12::WeakPtr::<dxgi::IDXGIAdapter1>::null();
                unsafe {
                    HRESULT(factory6.EnumAdapterByGpuPreference(
                        adapter_index,
                        dxgi1_6::DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE,
                        &dxgi::IDXGIAdapter1::uuidof(),
                        adapter.mut_void(),
                    ))
                }
                .ok()?;

                let mut desc = DXGI_ADAPTER_DESC1::default();
                unsafe { HRESULT(adapter.GetDesc1(&mut desc)) }.ok()?;
                let has_hw_acceleration = desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE == 0;
                let description = &desc.Description;
                let description_len = description.iter().take_while(|ch| **ch != 0).count();
                let description = AdapterDescription {
                    device_id: desc.DeviceId,
                    vendor_id: desc.VendorId,
                    description: HSTRING::from_wide(&description[0..description_len]).to_string(),
                    has_hw_acceleration,
                };
                Ok(AdapterDetails {
                    adapter: Adapter::new(self.lib_d3d12.clone(), unsafe {
                        DxgiAdapter::from_adapter1(adapter)
                    }),
                    description,
                })
            })
            .take_while(|result| match &result {
                Err(Error::WindowsError(err)) => err.code().0 != winerror::DXGI_ERROR_NOT_FOUND,
                _ => true,
            })
            .collect()
    }
}
