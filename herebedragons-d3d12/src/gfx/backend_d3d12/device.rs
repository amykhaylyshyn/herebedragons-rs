use super::Backend;

pub struct Device {
    device: d3d12::Device,
}

impl Device {
    pub fn new(device: d3d12::Device) -> Self {
        Self { device }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe { self.device.destroy() };
    }
}

impl crate::gfx::Device<Backend> for Device {}
