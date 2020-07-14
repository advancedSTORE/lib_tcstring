#![warn(clippy::all)]
#[macro_use]
mod macros;
mod decode;

pub use decode::model::{
    PublisherRestriction, PublisherRestrictionType, TCModel, TCModelV1, TCModelV2, TCSDecodeError,
    VendorSet,
};
