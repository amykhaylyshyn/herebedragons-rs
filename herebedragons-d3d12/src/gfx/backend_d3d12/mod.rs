mod instance;

pub use instance::*;

#[derive(Clone)]
pub struct Backend;

impl crate::gfx::Backend for Backend {
    type Instance = Instance;
}
