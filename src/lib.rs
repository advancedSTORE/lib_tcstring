#![warn(clippy::all)]

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

#[macro_use]
mod macros;
mod decode;

pub use decode::model::{
    PublisherRestriction, PublisherRestrictionType, TCModel, TCModelV1, TCModelV2, TCSDecodeError,
    VendorSet,
};
