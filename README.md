![Crates.io license](https://img.shields.io/crates/l/lib_tcstring?style=flat-square)
![Crates.io version](https://img.shields.io/crates/v/lib_tcstring?style=flat-square)
![GitHub Workflow Status](https://img.shields.io/github/workflow/status/advancedSTORE/lib_tcstring/CI?style=flat-square)
![dependency status for latest release](https://img.shields.io/librariesio/release/cargo/lib_tcstring?style=flat-square)

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
lib_tcstring = "0.4.0"
```

Code

```rust
use std::convert::TryFrom;

fn main() {
    let tc_model_v2 = lib_tcstring::TcModel::try_from("COvFyGBOvFyGBAbAAAENAPCAAOAAAAAAAAAAAEEUACCKAAA");

    println!("{:?}", tc_model_v2);
}
```
