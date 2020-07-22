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

/// Errors that can occur while decoding the TCString
#[derive(Clone, Debug, PartialEq)]
pub enum TcsError {
    /// TCString doesn't have enough bits
    InsufficientLength,
    /// TCString contains an invalid or unsupported version
    UnsupportedVersion,
    /// TCString isn't valid base64
    InvalidUrlSafeBase64(DecodeError),
    /// TCString contains an invalid offset for string creation
    InvalidAlphabetOffset,
    /// TCString contains an invalid section definition
    InvalidSectionDefinition,
    /// TCString contains an invalid segment definition
    InvalidSegmentDefinition,
    /// TCString contains an unkown range section definition
    UnexpectedRangeSection,
}

impl Display for TcsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TcsError::InsufficientLength => write!(f, "{}", INSUFFICIENT_LENGTH),
            TcsError::UnsupportedVersion => write!(f, "{}", UNSUPPORTED_VERSION),
            TcsError::InvalidUrlSafeBase64(decode_error) => {
                write!(f, "{}: {}", INVALID_URL_SAFE_BASE64, decode_error)
            }
            TcsError::InvalidAlphabetOffset => write!(f, "{}", INVALID_ALPHABET_OFFSET),
            TcsError::InvalidSectionDefinition => write!(f, "{}", INVALID_SECTION_DEFINITION),
            TcsError::InvalidSegmentDefinition => write!(f, "{}", INVALID_SEGMENT_DEFINITION),
            TcsError::UnexpectedRangeSection => write!(f, "{}", UNEXPECTED_RANGE_SECTION),
        }
    }
}

impl Error for TcsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            TcsError::InvalidUrlSafeBase64(err) => Some(err),
            _ => None,
        }
    }
}
