use d3d12::D3DResult;
use windows::core::{Error, HRESULT};

pub trait IntoResult {
    type Type;
    type Error;

    fn into_result(self) -> Result<Self::Type, Self::Error>;
}

impl<T> IntoResult for D3DResult<T> {
    type Type = T;
    type Error = Error;

    fn into_result(self) -> Result<Self::Type, Self::Error> {
        let _ = HRESULT(self.1).ok()?;
        Ok(self.0)
    }
}
