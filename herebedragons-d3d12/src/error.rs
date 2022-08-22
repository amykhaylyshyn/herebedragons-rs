use derive_more::{Display, From};

#[derive(Debug, Display)]
pub enum RenderDeviceError {
    #[display(fmt = "Load library error")]
    LoadLibraryError,
    #[display(fmt = "DxgiFactory{} is not available", "_0")]
    DXGIFactoryNotAvaiable(usize),
}

impl std::error::Error for RenderDeviceError {}

#[derive(Debug, Display, From)]
pub enum Error {
    RenderDeviceError(RenderDeviceError),
    WindowsError(windows::core::Error),
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::RenderDeviceError;

    #[test]
    fn test_render_device_error_display() {
        assert_eq!(
            format!("{}", RenderDeviceError::LoadLibraryError),
            "Load library error".to_string()
        );
        assert_eq!(
            format!("{}", RenderDeviceError::DXGIFactoryNotAvaiable(6)),
            "DxgiFactory6 is not available".to_string()
        );
    }
}
