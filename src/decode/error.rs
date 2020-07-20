use base64::DecodeError;
use std::{
    error::Error,
    fmt::{Display, Formatter},
};

pub const INSUFFICIENT_LENGTH: &str = "ERR_INSUFFICIENT_LENGTH";
pub const UNSUPPORTED_VERSION: &str = "ERR_UNSUPPORTED_VERSION";
pub const INVALID_URL_SAFE_BASE64: &str = "ERR_INVALID_URL_SAFE_BASE64";
pub const INVALID_ALPHABET_OFFSET: &str = "ERR_INVALID_ALPHABET_OFFSET";
pub const INVALID_SECTION_DEFINITION: &str = "ERR_INVALID_SECTION_DEFINITION";
pub const INVALID_SEGMENT_DEFINITION: &str = "ERR_INVALID_SEGMENT_DEFINITION";
pub const UNEXPECTED_RANGE_SECTION: &str = "ERR_UNEXPECTED_RANGE_SECTION";

#[derive(Debug, PartialEq)]
pub enum TcfError {
    InsufficientLength,
    UnsupportedVersion,
    InvalidUrlSafeBase64(DecodeError),
    InvalidAlphabetOffset,
    InvalidSectionDefinition,
    InvalidSegmentDefinition,
    UnexpectedRangeSection,
}

impl Display for TcfError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TcfError::InsufficientLength => write!(f, "{}", INSUFFICIENT_LENGTH),
            TcfError::UnsupportedVersion => write!(f, "{}", UNSUPPORTED_VERSION),
            TcfError::InvalidUrlSafeBase64(decode_error) => {
                write!(f, "{}: {}", INVALID_URL_SAFE_BASE64, decode_error)
            }
            TcfError::InvalidAlphabetOffset => write!(f, "{}", INVALID_ALPHABET_OFFSET),
            TcfError::InvalidSectionDefinition => write!(f, "{}", INVALID_SECTION_DEFINITION),
            TcfError::InvalidSegmentDefinition => write!(f, "{}", INVALID_SEGMENT_DEFINITION),
            TcfError::UnexpectedRangeSection => write!(f, "{}", UNEXPECTED_RANGE_SECTION),
        }
    }
}

impl Error for TcfError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            TcfError::InvalidUrlSafeBase64(err) => Some(err),
            _ => None,
        }
    }
}
