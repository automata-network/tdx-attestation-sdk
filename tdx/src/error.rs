use coco_provider::error::CocoError;

pub type Result<T> = std::result::Result<T, TdxError>;

#[derive(Debug, Clone, Eq, PartialEq, thiserror::Error)]
pub enum TdxError {
    #[error("Anyhow: {0}")]
    Anyhow(String),
    #[error("ConfigOptions: {0}")]
    ConfigOptions(String),
    #[error("Cpu: {0}")]
    Cpu(String),
    #[error("Dcap: {0}")]
    Dcap(String),
    #[error("Firmware: {0}")]
    Firmware(String),
    #[error("Http: {0}")]
    Http(String),
    #[error("IO: {0}")]
    IO(String),
    #[error("SSL: {0}")]
    SSL(String),
    #[error("Tpm: {0}")]
    Tpm(String),
    #[error("X509: {0}")]
    X509(String),
    #[error("Unknown")]
    Unknown,
}

impl From<CocoError> for TdxError {
    fn from(err: CocoError) -> Self {
        TdxError::Firmware(format!("{:?}", err))
    }
}

impl From<base64_url::base64::DecodeError> for TdxError {
    fn from(err: base64_url::base64::DecodeError) -> Self {
        TdxError::IO(format!("{:?}", err))
    }
}

impl From<std::io::Error> for TdxError {
    fn from(err: std::io::Error) -> Self {
        TdxError::IO(format!("{:?}", err))
    }
}

impl From<ureq::Error> for TdxError {
    fn from(err: ureq::Error) -> Self {
        TdxError::Http(format!("{:?}", err))
    }
}

impl From<anyhow::Error> for TdxError {
    fn from(err: anyhow::Error) -> Self {
        TdxError::Anyhow(err.to_string())
    }
}

impl From<std::str::Utf8Error> for TdxError {
    fn from(err: std::str::Utf8Error) -> Self {
        TdxError::IO(format!("{:?}", err))
    }
}
