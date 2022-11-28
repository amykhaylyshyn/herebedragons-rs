use std::sync::Arc;

use d3d12::{DxgiAdapter, DxgiFactory, FactoryCreationFlags};
use raw_window_handle::RawWindowHandle;
use winapi::{
    shared::{
        dxgi::{self, DXGI_ADAPTER_DESC1, DXGI_ADAPTER_FLAG_SOFTWARE},
        dxgi1_6, dxgiformat, dxgitype, winerror,
    },
    um::dxgidebug,
    Interface,
};
use windows::core::{HRESULT, HSTRING};

use crate::{
    error::{Error, Result},
    gfx::{
        AdapterDescription, AdapterDetails, Format, NewInstanceOptions, ScopedResource,
        SwapChainOptions, SwapEffect,
    },
    hresult::IntoResult,
};

use super::{Adapter, BackendD3D12, Device, Queue, SwapChain};

struct DxgiFormat(u32);

impl From<Format> for DxgiFormat {
    fn from(format: Format) -> Self {
        match format {
            Format::R8G8B8A8UNorm => Self(dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM),
        }
    }
}

impl From<SwapEffect> for d3d12::SwapEffect {
    fn from(swap_effect: SwapEffect) -> Self {
        match swap_effect {
            SwapEffect::Discard => d3d12::SwapEffect::Discard,
            SwapEffect::Sequential => d3d12::SwapEffect::Sequential,
            SwapEffect::FlipSequential => d3d12::SwapEffect::FlipSequential,
            SwapEffect::FlipDiscard => d3d12::SwapEffect::FlipDiscard,
        }
    }
}

impl From<&SwapChainOptions> for d3d12::SwapchainDesc {
    fn from(options: &SwapChainOptions) -> Self {
        let format: DxgiFormat = options.format.into();
        Self {
            width: options.width,
            height: options.height,
            format: format.0,
            stereo: false,
            sample: d3d12::SampleDesc {
                count: options.sample_options.sample_count as u32,
                quality: 0,
            },
            buffer_usage: dxgitype::DXGI_USAGE_RENDER_TARGET_OUTPUT,
            buffer_count: options.backbuffer_count as _,
            scaling: d3d12::Scaling::Stretch,
            swap_effect: options.swap_effect.into(),
            alpha_mode: d3d12::AlphaMode::Unspecified,
            flags: 0,
        }
    }
}

pub struct Instance {
    lib_d3d12: Arc<d3d12::D3D12Lib>,
    _lib_dxgi: d3d12::DxgiLib,
    factory: d3d12::DxgiFactory,
}

impl Instance {
    pub fn new(options: NewInstanceOptions) -> Result<Self> {
        let lib_d3d12 = d3d12::D3D12Lib::new().map_err(|err| {
            log::error!("load d3d12 library error: {}", err);
            Error::LoadLibraryError
        })?;
        let lib_dxgi = d3d12::DxgiLib::new().map_err(|err| {
            log::error!("load dxgi library error: {}", err);
            Error::LoadLibraryError
        })?;

        let dbg_factory = if options.enable_debug_layer {
            let dbg_interface = lib_d3d12
                .get_debug_interface()
                .map_err(|err| {
                    log::error!("D3D get debug interface error: {}", err);
                    Error::LoadLibraryError
                })?
                .into_result()
                .map_err(|err| log::warn!("Direct3D debug device is not available: {}", err))
                .ok();
            if let Some(dbg_interface) = dbg_interface {
                unsafe { dbg_interface.EnableDebugLayer() };
            }

            let dxgi_info_queue = lib_dxgi
                .get_debug_interface1()
                .map_err(|err| {
                    log::error!("DXGI get debug interface error: {}", err);
                    Error::LoadLibraryError
                })?
                .into_result()
                .ok();
            if let Some(dxgi_info_queue) = dxgi_info_queue {
                let factory = unsafe {
                    DxgiFactory::from_factory4(
                        lib_dxgi
                            .create_factory2(FactoryCreationFlags::DEBUG)
                            .map_err(|err| {
                                log::error!("create dxgi factor2 error: {}", err);
                                Error::LoadLibraryError
                            })?
                            .into_result()?,
                    )
                };

                unsafe {
                    HRESULT(dxgi_info_queue.SetBreakOnSeverity(
                        dxgidebug::DXGI_DEBUG_ALL,
                        dxgidebug::DXGI_INFO_QUEUE_MESSAGE_SEVERITY_ERROR,
                        1,
                    ))
                    .ok()?;
                    HRESULT(dxgi_info_queue.SetBreakOnSeverity(
                        dxgidebug::DXGI_DEBUG_ALL,
                        dxgidebug::DXGI_INFO_QUEUE_MESSAGE_SEVERITY_CORRUPTION,
                        1,
                    ))
                    .ok()?;
                }

                Some(factory)
            } else {
                None
            }
        } else {
            None
        };

        let factory = if let Some(dbg_factory) = dbg_factory {
            dbg_factory
        } else {
            unsafe {
                DxgiFactory::from_factory1(
                    lib_dxgi
                        .create_factory1()
                        .map_err(|err| {
                            log::error!("load dxgi library error: {}", err);
                            Error::LoadLibraryError
                        })?
                        .into_result()?,
                )
            }
        };

        Ok(Self {
            lib_d3d12: Arc::new(lib_d3d12),
            _lib_dxgi: lib_dxgi,
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

impl crate::gfx::Instance<BackendD3D12> for Instance {
    fn new(options: NewInstanceOptions) -> Result<Self> {
        Self::new(options)
    }

    fn enumerate_adapters(&self) -> Result<Vec<AdapterDetails<BackendD3D12>>> {
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
                    adapter: Adapter::new(unsafe { DxgiAdapter::from_adapter1(adapter) }),
                    description,
                })
            })
            .take_while(|result| match &result {
                Err(Error::WindowsError(err)) => err.code().0 != winerror::DXGI_ERROR_NOT_FOUND,
                _ => true,
            })
            .collect()
    }

    fn create_device(&self, adapter: &Adapter) -> Result<ScopedResource<Instance, Device>> {
        let adapter = unsafe { adapter.raw().cast::<dxgi::IDXGIAdapter1>().into_result()? };
        let device = self
            .lib_d3d12
            .create_device(adapter, d3d12::FeatureLevel::L11_0)
            .map_err(|err| {
                log::error!("Create D3D12 error {}", err);
                Error::LoadLibraryError
            })?
            .into_result()?;
        Ok(ScopedResource::new(self, Device::new(device)))
    }

    fn create_swap_chain(
        &self,
        raw_window: &RawWindowHandle,
        queue: &Queue,
        options: &crate::gfx::SwapChainOptions,
    ) -> Result<ScopedResource<Instance, SwapChain>> {
        let raw_window = match raw_window {
            RawWindowHandle::Win32(raw_window) => raw_window,
            _ => panic!("d3d12 is supported only on windows"),
        };

        let desc = options.into();

        self.factory.create_swapchain(
            queue.queue.as_mut_ptr() as *mut _,
            raw_window.hwnd as *mut _,
            &desc,
        );
        Ok(ScopedResource::new(self, SwapChain::new()))
    }
}
