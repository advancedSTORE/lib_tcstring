# v0.5

* changed the edition from 2018 to 2021
* updated dependency [base64](https://crates.io/crates/base64) to version `0.22`

# v0.4.1

* updated dependency [base64](https://crates.io/crates/base64) to version `0.21`

# v0.4

* removed `TcModel` (obsolete since there is only one version, use `TcModelV2::try_from` instead)
* removed `TcModelV1`
* removed `VendorSet`
* removed `::new()` implementation for the following `struct`s (use `::default()` instead)
    * `TcModelV2`
    * `PublisherRestriction`

# v0.3

* renamed `TCModel` to `TcModel`
* renamed `TCModelV1` to `TcModelV1`
* renamed `TCModelV2` to `TcModelV2`
* `TcModelV2`
    * renamed `TCSegment` to `TcSegment`
    * renamed `PublisherTC` to `PublisherTc`

# v0.2

* `TCModelV1`
    * renamed `purpose_consents` to `purposes_consent`
* `TCModelV2`
    * renamed the following fields
        * `purpose_consents` -> `purposes_consent`
        * `purpose_li_transparency` -> `purposes_li_transparency`
        * `vendor_consents` -> `vendors_consent`
        * `vendor_li_consents` -> `vendors_li_consent`
