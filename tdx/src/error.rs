use std::fmt::Display;

use coco_provider::error::CocoError;

pub type Result<T> = std::result::Result<T, TdxError>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TdxError {
    Anyhow(String),
    ConfigOptions(String),
    Cpu(String),
    Dcap(String),
    Firmware(String),
    Http(String),
    IO(String),
    SSL(String),
    Tpm(String),
    X509(String),
    Unknown,
}

impl Display for TdxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TdxError::Anyhow(err) => write!(f, "Anyhow: {}", err),
            TdxError::ConfigOptions(err) => write!(f, "ConfigOptions: {}", err),
            TdxError::Cpu(err) => write!(f, "Cpu: {}", err),
            TdxError::Dcap(err) => write!(f, "Dcap: {}", err),
            TdxError::Firmware(err) => write!(f, "Firmware: {}", err),
            TdxError::Http(err) => write!(f, "Http: {}", err),
            TdxError::IO(err) => write!(f, "IO: {}", err),
            TdxError::SSL(err) => write!(f, "SSL: {}", err),
            TdxError::Tpm(err) => write!(f, "Tpm: {}", err),
            TdxError::X509(err) => write!(f, "X509: {}", err),
            TdxError::Unknown => write!(f, "Unknown"),
        }
    }
}

impl std::error::Error for TdxError {}

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

impl From<&str> for TdxError {
    fn from(err: &str) -> Self {
        TdxError::Firmware(err.to_string())
    }
}

impl From<anyhow::Error> for TdxError {
    fn from(err: anyhow::Error) -> Self {
        TdxError::Anyhow(err.to_string())
    }
}
