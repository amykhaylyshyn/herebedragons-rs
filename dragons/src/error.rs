use derive_more::{Display, From};

#[derive(Debug, Display, From)]
pub enum Error {
    #[display(fmt = "Load library error")]
    LoadLibraryError,
    WindowsError(windows::core::Error),
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
