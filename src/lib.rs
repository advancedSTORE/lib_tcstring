//! # TCF String library which (currently only) decodes a given TCString
//!
//! NOTE: This is not an official IAB library
//!
//! General usage
//! ```rust,edition2018
//! use std::convert::TryFrom;
//! // returns a Result which contains a TcModel V2
//! let tc_model_v2 = lib_tcstring::TcModel::try_from("COvFyGBOvFyGBAbAAAENAPCAAOAAAAAAAAAAAEEUACCKAAA");
//! // returns a Result which contains a TcModel V1
//! let tc_model_v1 = lib_tcstring::TcModel::try_from("BOEFEAyOEFEAyAHABDENAI4AAAB9vABAASA");
//! ```
//!
//! If it's possible to know which TcModel version is used write it like this:
//! ```rust,edition2018
//! use std::convert::TryFrom;
//! let tc_model_v2 = lib_tcstring::TcModelV2::try_from("COvFyGBOvFyGBAbAAAENAPCAAOAAAAAAAAAAAEEUACCKAAA");
//! let tc_model_v1 = lib_tcstring::TcModelV1::try_from("BOEFEAyOEFEAyAHABDENAI4AAAB9vABAASA");
//! ```

#![warn(clippy::all)]
#![doc(html_root_url = "https://docs.rs/lib_tcstring/0.3.0")]
#![warn(missing_docs)]
#![forbid(unsafe_code)]

#[cfg(feature = "serde")]
extern crate serde;

#[macro_use]
mod macros;
mod decode;

pub use decode::{
    error::TcsError,
    model::{
        PublisherRestriction, PublisherRestrictionType, TcModel, TcModelV1, TcModelV2, VendorSet,
    },
};

mod tests {
    #[test]
    fn test_readme_deps() {
        version_sync::assert_markdown_deps_updated!("README.md");
    }

    #[test]
    fn test_html_root_url() {
        version_sync::assert_html_root_url_updated!("src/lib.rs");
    }
}
