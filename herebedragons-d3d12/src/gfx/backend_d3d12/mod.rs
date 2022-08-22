mod adapter;
mod device;
mod instance;

pub use adapter::*;
pub use device::*;
pub use instance::*;

#[derive(Clone)]
pub struct Backend;

impl crate::gfx::Backend for Backend {
    type Instance = Instance;
    type Adapter = Adapter;
    type Device = Device;
}
