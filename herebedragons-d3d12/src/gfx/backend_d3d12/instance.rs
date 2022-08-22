use std::sync::Arc;

use d3d12::{DxgiFactory, FactoryCreationFlags};
use winapi::{
    shared::{
        dxgi::{self, DXGI_ADAPTER_DESC1, DXGI_ADAPTER_FLAG_SOFTWARE},
        dxgi1_6,
    },
    Interface,
};
use windows::core::HRESULT;

use crate::{
    error::{Error, RenderDeviceError},
    hresult::IntoResult,
};

use super::Backend;

pub struct Instance {
    lib_d3d12: Arc<d3d12::D3D12Lib>,
    lib_dxgi: d3d12::DxgiLib,
    factory: d3d12::DxgiFactory,
}

impl Instance {
    pub fn new() -> Result<Self, Error> {
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
    fn enumerate_adapters(&self) {
        let factory6 = self
            .factory
            .as_factory6()
            .expect("factory6 is not available");
        let adapters = (0..)
            .map(|adapter_index| -> Result<_, Error> {
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
                if desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE != 0 {
                    Ok(None)
                } else {
                    Ok(Some(adapter))
                }
            })
            .find_map(|result| match result {
                Ok(Some(adapter)) => Some(Ok(adapter)),
                Err(err) => Some(Err(err)),
                _ => None,
            });
    }
}
