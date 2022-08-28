mod error;
mod gfx;
mod hresult;
mod renderer;

use crate::error::Result;
use dotenv::dotenv;
use gfx::backend_d3d12::BackendD3D12;
use renderer::render_main;

fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    render_main::<BackendD3D12>(3)
}
