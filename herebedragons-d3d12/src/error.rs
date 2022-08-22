use derive_more::{Display, From};

#[derive(Debug, Display)]
pub enum RenderDeviceError {
    #[display(fmt = "Load library error")]
    LoadLibraryError,
}

impl std::error::Error for RenderDeviceError {}

#[derive(Debug, Display, From)]
pub enum Error {
    RenderDeviceError(RenderDeviceError),
    WindowsError(windows::core::Error),
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
