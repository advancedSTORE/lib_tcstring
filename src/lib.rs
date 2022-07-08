//! # TCF String library which (currently only) decodes a given TCString
//!
//! NOTE: This is not an official IAB library
//!
//! General usage
//! ```rust,edition2018
//! use std::convert::TryFrom;
//! // returns a Result which contains a TcModel V2
//! let tc_model_v2 = lib_tcstring::TcModelV2::try_from("COvFyGBOvFyGBAbAAAENAPCAAOAAAAAAAAAAAEEUACCKAAA");
//! ```

#![warn(clippy::all)]
#![doc(html_root_url = "https://docs.rs/lib_tcstring/0.4.0")]
#![warn(missing_docs)]
#![forbid(unsafe_code)]

#[cfg(feature = "serde")]
extern crate serde;

pub use decode::{
    error::TcsError,
    model::{PublisherRestriction, PublisherRestrictionType, TcModelV2},
};

#[macro_use]
mod macros;
mod decode;

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
