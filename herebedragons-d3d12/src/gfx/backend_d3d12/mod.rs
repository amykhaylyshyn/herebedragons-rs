mod adapter;
mod instance;

pub use adapter::*;
pub use instance::*;

#[derive(Clone)]
pub struct Backend;

impl crate::gfx::Backend for Backend {
    type Instance = Instance;
    type Adapter = Adapter;
}
