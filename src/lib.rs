//! # `iab_tcstring` is an TCF String library which will (currently only) decode a given TCString
//!
//! NOTE: This is not an official IAB library
//!
//! General usage
//! ```edition2018
//! use std::convert::TryFrom;
//! // will return an Result which contains an TCModel V2
//! let tc_model_v2 = iab_tcstring::TCModel::try_from("COvFyGBOvFyGBAbAAAENAPCAAOAAAAAAAAAAAEEUACCKAAA");
//! // will return an Result which contains an TCModel V1
//! let tc_model_v1 = iab_tcstring::TCModel::try_from("BOEFEAyOEFEAyAHABDENAI4AAAB9vABAASA");
//! ```
//!
//! If it's possible to know which TCModel version is used you can instead write it like this:
//! ```edition2018
//! use std::convert::TryFrom;
//! let tc_model_v2 = iab_tcstring::TCModelV2::try_from("COvFyGBOvFyGBAbAAAENAPCAAOAAAAAAAAAAAEEUACCKAAA");
//! let tc_model_v1 = iab_tcstring::TCModelV1::try_from("BOEFEAyOEFEAyAHABDENAI4AAAB9vABAASA");
//! ```

#![warn(clippy::all)]
#![doc(html_root_url = "https://docs.rs/iab_tcstring/0.1.0")]
#![warn(missing_docs)]

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

#[macro_use]
mod macros;
mod decode;

pub use decode::model::{
    PublisherRestriction, PublisherRestrictionType, TCModel, TCModelV1, TCModelV2, VendorSet,
};

pub use decode::error::TcsError;

mod tests {
    #[test]
    fn test_html_root_url() {
        version_sync::assert_html_root_url_updated!("src/lib.rs");
    }
}
