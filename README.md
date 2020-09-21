![CI](https://github.com/advancedSTORE/lib_tcstring/workflows/CI/badge.svg)

# IAB TCString library
A utility library to work with the IAB TCF v1 & v2 strings.

**NOTE**: This is not an official IAB library

**NOTE**: Currently only TCString decoding is implemented

## Documentation
Please go to [docs.rs/lib_tcstring](https://docs.rs/lib_tcstring)

## Changelog
For major (or breaking) version changes see [CHANGELOG.md](./CHANGELOG.md)

## Example
`Cargo.toml`
```toml
[dependencies]
lib_tcstring = "0.2.2"
```

Code
```rust
use std::convert::TryFrom;

let tc_model_v2 = lib_tcstring::TCModel::try_from("COvFyGBOvFyGBAbAAAENAPCAAOAAAAAAAAAAAEEUACCKAAA");
```