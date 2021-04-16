/// `TcModel` serves as a convenience wrapper to parse a given TCString
/// without checking the version on the calling side
///
/// ```rust,edition2018
/// use std::convert::TryFrom;
/// // will return a Result which contains the variant for the TCString version or an Error
/// // if the TCString could not be parsed or the TCString includes an unsupported version
/// let tc_model = lib_tcstring::TcModel::try_from("COvFyGBOvFyGBAbAAAENAPCAAOAAAAAAAAAAAEEUACCKAAA");
/// ```
#[derive(PartialEq, Clone, Debug)]
pub enum TcModel {
    /// Contains a reference to the [`TcModelV1`]
    ///
    /// [`TcModelV1`]: struct.TcModelV1.html
    V1(Box<TcModelV1>),
    /// Contains a reference to the [`TcModelV2`]
    ///
    /// [`TcModelV2`]: struct.TcModelV2.html
    V2(Box<TcModelV2>),
}

/// Contains restriction types as defined in [`Vendor Consent String Format V2 Core String`]
///
/// [`Vendor Consent String Format V2 Core String`]: https://github.com/InteractiveAdvertisingBureau/GDPR-Transparency-and-Consent-Framework/blob/81a3b9ed1545148be380b4408e6361cd2294446d/TCFv2/IAB%20Tech%20Lab%20-%20Consent%20string%20and%20vendor%20list%20formats%20v2.md#the-core-string
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Clone, PartialOrd, Hash, Debug)]
pub enum PublisherRestrictionType {
    /// Purpose Flatly Not Allowed by Publisher
    NotAllowed,
    /// Specifies that vendors need to have consent
    RequireConsent,
    /// Specifies that vendors need to have "Legitimate Interest"
    RequireLegitimateInterest,
    /// Should not be used
    Undefined,
}

/// `VendorSet` contains a list of vendors which are either allowed or blocked
/// based on the [`is_blocklist`] field
///
/// [`is_blocklist`]: struct.VendorSet.html#structfield.is_blocklist
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Clone, PartialOrd, Hash, Debug, Default)]
pub struct VendorSet {
    /// `is_blocklist` defines if [`list`] is an blocklist or an allowlist
    ///
    /// [`list`]: struct.VendorSet.html#structfield.list
    pub is_blocklist: bool,
    /// List of vendors which are either allowed or blocked based on the [`is_blocklist`]
    ///
    /// [`is_blocklist`]: struct.VendorSet.html#structfield.is_blocklist
    pub list: Vec<u16>,
}

/// `TcModelV1` contains all relevant fields specified in the [`Vendor Consent String Format V1`]
/// except for the `Version` field which is omitted
///
/// Field mapping
/// * `Created` -> [`created_at`]
/// * `LastUpdated` -> [`updated_at`]
/// * `CmpId` -> [`cmp_id`]
/// * `CmpVersion` -> [`cmp_version`]
/// * `ConsentScreen` -> [`consent_screen`]
/// * `ConsentLanguage` -> [`consent_lang`]
/// * `VendorListVersion` -> [`vendor_list_version`]
/// * `PurposesAllowed` -> [`purposes_consent`]
/// * `VendorConsents` -> [`vendors`]
///
/// ```rust,edition2018
/// use std::convert::TryFrom;
/// // will return a Result which contains either the TcModel or an Error
/// // if the TCString could not be parsed or the TCString includes an unsupported version
/// let tc_model = lib_tcstring::TcModelV1::try_from("BOEFEAyOEFEAyAHABDENAI4AAAB9vABAASA");
/// ```
///
/// [`created_at`]: struct.TcModelV1.html#structfield.created_at
/// [`updated_at`]: struct.TcModelV1.html#structfield.updated_at
/// [`cmp_id`]: struct.TcModelV1.html#structfield.cmp_id
/// [`cmp_version`]: struct.TcModelV1.html#structfield.cmp_version
/// [`consent_screen`]: struct.TcModelV1.html#structfield.consent_screen
/// [`consent_lang`]: struct.TcModelV1.html#structfield.consent_lang
/// [`vendor_list_version`]: struct.TcModelV1.html#structfield.vendor_list_version
/// [`purposes_consent`]: struct.TcModelV1.html#structfield.purposes_consent
/// [`vendors`]: struct.TcModelV1.html#structfield.vendors
/// [`Vendor Consent String Format V1`]: https://github.com/InteractiveAdvertisingBureau/GDPR-Transparency-and-Consent-Framework/blob/ab7e3dcf8c493c743cac87c9bce49c16fc0523e4/Consent%20string%20and%20vendor%20list%20formats%20v1.1%20Final.md#vendor-consent-string-format-
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Clone, PartialOrd, Hash, Debug, Default)]
pub struct TcModelV1 {
    /// Epoch milliseconds when consent string was first created
    pub created_at: u64,
    /// Epoch milliseconds when consent string was last updated
    pub updated_at: u64,
    /// Consent Manager Provider ID that last updated the consent string
    pub cmp_id: u16,
    /// Consent Manager Provider version
    pub cmp_version: u16,
    /// Screen number in the CMP where consent was given
    pub consent_screen: u8,
    /// [`ISO 639-1`] language code in which the CMP UI was presented
    ///
    /// [`ISO 639-1`]: https://en.wikipedia.org/wiki/ISO_639-1
    pub consent_lang: String,
    /// Version of vendor list used in most recent consent string update
    pub vendor_list_version: u16,
    /// List of permitted purposes
    pub purposes_consent: Vec<u8>,
    /// List of allowed or blocked vendors
    ///
    /// See [`VendorSet`] for more details
    ///
    /// [`VendorSet`]: struct.VendorSet.html
    pub vendors: VendorSet,
}

/// `TcModelV2` contains all relevant fields specified in the [`Vendor Consent String Format V2`]
/// except for the `Version` field which is omitted
///
/// Note that the "Core String", "Disclosed Vendors", "Allowed Vendors" and "Publisher TC" segments are mapped into fields
///
/// "Core String" field mapping
/// * `Created` -> [`created_at`]
/// * `LastUpdated` -> [`updated_at`]
/// * `CmpId` -> [`cmp_id`]
/// * `CmpVersion` -> [`cmp_version`]
/// * `ConsentScreen` -> [`consent_screen`]
/// * `ConsentLanguage` -> [`consent_language`]
/// * `VendorListVersion` -> [`vendor_list_version`]
/// * `TcfPolicyVersion` -> [`tcf_policy_version`]
/// * `IsServiceSpecific` -> [`is_service_specific`]
/// * `UseNonStandardStacks` -> [`use_non_standard_stacks`]
/// * `SpecialFeatureOptIns` -> [`special_feature_opt_ins`]
/// * `PurposesConsent` -> [`purposes_consent`]
/// * `PurposesLITransparancy` -> [`purposes_li_transparency`]
/// * `PurposeOneTreatment` -> [`purpose_one_treatment`]
/// * `PublisherCC` -> [`publisher_country_code`]
/// * `VendorsConsent` -> [`vendors_consent`]
/// * `VendorsLIConsent` -> [`vendors_li_consent`]
/// * `PublisherRestrictions` -> [`publisher_restrictions`]
///
/// "Disclosed Vendors" segment is mapped by the [`disclosed_vendors`] field
///
/// "Allowed Vendors" segment is mapped by the [`allowed_vendors`] field
///
/// "Publisher TC" field mapping
/// * `PubPurposesConsent` -> [`publisher_purposes_consent`]
/// * `PubPurposesLITransparency` -> [`publisher_purposes_li_transparency`]
/// * `CustomPurposesConsent` -> [`custom_purposes_consent`]
/// * `CustomPurposesLITransparency` -> [`custom_purposes_li_transparency`]
///
/// ```rust,edition2018
/// use std::convert::TryFrom;
/// // will return a Result which contains either the TcModel or an Error
/// // if the TCString could not be parsed or the TCString includes an unsupported version
/// let tc_model = lib_tcstring::TcModelV2::try_from("COvFyGBOvFyGBAbAAAENAPCAAOAAAAAAAAAAAEEUACCKAAA");
/// ```
///
/// [`created_at`]: struct.TcModelV2.html#structfield.created_at
/// [`updated_at`]: struct.TcModelV2.html#structfield.updated_at
/// [`cmp_id`]: struct.TcModelV2.html#structfield.cmp_id
/// [`cmp_version`]: struct.TcModelV2.html#structfield.cmp_version
/// [`consent_screen`]: struct.TcModelV2.html#structfield.consent_screen
/// [`consent_language`]: struct.TcModelV2.html#structfield.consent_language
/// [`vendor_list_version`]: struct.TcModelV2.html#structfield.vendor_list_version
/// [`tcf_policy_version`]: struct.TcModelV2.html#structfield.tcf_policy_version
/// [`is_service_specific`]: struct.TcModelV2.html#structfield.is_service_specific
/// [`use_non_standard_stacks`]: struct.TcModelV2.html#structfield.use_non_standard_stacks
/// [`special_feature_opt_ins`]: struct.TcModelV2.html#structfield.special_feature_opt_ins
/// [`purposes_consent`]: struct.TcModelV2.html#structfield.purposes_consent
/// [`purposes_li_transparency`]: struct.TcModelV2.html#structfield.purposes_li_transparency
/// [`purpose_one_treatment`]: struct.TcModelV2.html#structfield.purpose_one_treatment
/// [`publisher_country_code`]: struct.TcModelV2.html#structfield.publisher_country_code
/// [`vendors_consent`]: struct.TcModelV2.html#structfield.vendors_consent
/// [`vendors_li_consent`]: struct.TcModelV2.html#structfield.vendors_li_consent
/// [`publisher_restrictions`]: struct.TcModelV2.html#structfield.publisher_restrictions
/// [`disclosed_vendors`]: struct.TcModelV2.html#structfield.disclosed_vendors
/// [`allowed_vendors`]: struct.TcModelV2.html#structfield.allowed_vendors
/// [`publisher_purposes_consent`]: struct.TcModelV2.html#structfield.publisher_purposes_consent
/// [`publisher_purposes_li_transparency`]: struct.TcModelV2.html#structfield.publisher_purposes_li_transparency
/// [`custom_purposes_consent`]: struct.TcModelV2.html#structfield.custom_purposes_consent
/// [`custom_purposes_li_transparency`]: struct.TcModelV2.html#structfield.custom_purposes_li_transparency
/// [`Vendor Consent String Format V2`]: https://github.com/InteractiveAdvertisingBureau/GDPR-Transparency-and-Consent-Framework/blob/81a3b9ed1545148be380b4408e6361cd2294446d/TCFv2/IAB%20Tech%20Lab%20-%20Consent%20string%20and%20vendor%20list%20formats%20v2.md#tc-string-format
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Clone, PartialOrd, Hash, Debug, Default)]
pub struct TcModelV2 {
    /// Epoch milliseconds when this TC String was first created
    pub created_at: u64,
    /// Epoch milliseconds when TC String was last updated
    pub updated_at: u64,
    /// Consent Management Platform ID that last updated the TC String
    pub cmp_id: u16,
    /// Consent Management Platform version of the CMP that last updated this TC String
    pub cmp_version: u16,
    /// CMP Screen number at which consent was given for a user with the CMP that last updated this TC String
    pub consent_screen: u8,
    /// [`ISO 639-1`] language code in which the CMP UI was presented
    ///
    /// [`ISO 639-1`]: https://en.wikipedia.org/wiki/ISO_639-1
    pub consent_language: String,
    /// Version of the global vendor list used to create this TC String
    pub vendor_list_version: u16,
    /// Version of the corresponding field in the global vendor list that was used for obtaining consent
    pub tcf_policy_version: u16,
    /// Whether the signals encoded in this TC String were from service-specific storage (`true`) or shared storage (`false`)
    pub is_service_specific: bool,
    /// `true` means that a CMP is using customized Stack descriptions and not the standard stack descriptions
    /// defined in the [`Policies`] (Appendix A, Section E)
    ///
    /// `false` means standard stacks were used
    ///
    /// [`Policies`]: https://iabeurope.eu/iab-europe-transparency-consent-framework-policies/#___E_Stacks__
    pub use_non_standard_stacks: bool,
    /// List of opted-in "Special Features"
    ///
    /// "Special Features" are numerically identified in the global vendor list separately from normal features
    pub special_feature_opt_ins: Vec<u8>,
    /// List of allowed purposes
    pub purposes_consent: Vec<u8>,
    /// List of allowed "Legitimate Interest" purposes
    pub purposes_li_transparency: Vec<u8>,
    /// `true` means "Purpose 1" was not disclosed
    ///
    /// `false` means "Purpose 1" was disclosed commonly as consent
    pub purpose_one_treatment: bool,
    /// [`ISO 3166-1 alpha-2`] code
    ///
    /// [`ISO 3166-1 alpha-2`]: https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2
    pub publisher_country_code: String,
    /// List of allowed vendors
    pub vendors_consent: Vec<u16>,
    /// List of vendors "Legitimate Interest" disclosures
    pub vendors_li_consent: Vec<u16>,
    /// List of publisher restrictions on a per purpose basis
    ///
    /// See [`PublisherRestriction`] for more details
    ///
    /// [`PublisherRestriction`]: struct.PublisherRestriction.html
    pub publisher_restrictions: Vec<PublisherRestriction>,
    /// List of vendors that have been disclosed to a given user by a CMP
    pub disclosed_vendors: Vec<u16>,
    /// List of vendors the publisher permits to use OOB legal bases
    pub allowed_vendors: Vec<u16>,
    /// List of purposes which are established on the legal basis of consent, for the publisher
    pub publisher_purposes_consent: Vec<u8>,
    /// List of purposes which are established on the legal basis of "Legitimate Interest" and the user has not exercised their “Right to Object”
    pub publisher_purposes_li_transparency: Vec<u8>,
    /// List of allowed custom purposes, for the publisher
    pub custom_purposes_consent: Vec<u8>,
    /// List of custom purposes which are are established on the legal basis of "Legitimate Interest"
    pub custom_purposes_li_transparency: Vec<u8>,
}

/// Publisher restriction which overrides the specificed purpose
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Clone, PartialOrd, Hash, Debug, Default)]
pub struct PublisherRestriction {
    /// ID of publisher restricted purpose
    pub purpose_id: u8,
    /// publisher restriction for this purpose, see [`PublisherRestrictionType`] for more details
    ///
    /// [`PublisherRestrictionType`]: enum.PublisherRestrictionType.html
    pub restriction_type: PublisherRestrictionType,
    /// List of relevant vendors
    pub vendor_list: Vec<u16>,
}

#[cfg_attr(test, derive(Debug))]
pub(crate) enum RangeSectionType {
    Vendor(Vec<u16>),
    VendorLegitimateInterest(Vec<u16>),
    PublisherRestriction(Vec<PublisherRestriction>),
}

#[cfg_attr(test, derive(Debug))]
pub(crate) struct TcSegment {
    pub disclosed_vendors: Option<Vec<u16>>,
    pub allowed_vendors: Option<Vec<u16>>,
    pub publisher_tc: Option<PublisherTc>,
}

#[cfg_attr(test, derive(Debug))]
pub(crate) struct RangeSection {
    pub last_bit: usize,
    pub value: RangeSectionType,
}

#[cfg_attr(test, derive(Debug))]
pub(crate) struct PublisherTc {
    pub publisher_purposes_consent: Vec<u8>,
    pub publisher_purposes_li_transparency: Vec<u8>,
    pub custom_purposes_consent: Vec<u8>,
    pub custom_purposes_li_transparency: Vec<u8>,
}

impl Default for PublisherTc {
    fn default() -> Self {
        Self {
            custom_purposes_consent: vec![],
            custom_purposes_li_transparency: vec![],
            publisher_purposes_consent: vec![],
            publisher_purposes_li_transparency: vec![],
        }
    }
}

impl Default for PublisherRestrictionType {
    fn default() -> Self {
        Self::Undefined
    }
}

impl VendorSet {
    #[allow(dead_code)]
    fn new() -> Self {
        Self::default()
    }
}

impl TcModelV1 {
    #[allow(dead_code)]
    fn new() -> Self {
        Self::default()
    }
}

impl TcModelV2 {
    #[allow(dead_code)]
    fn new() -> Self {
        Self::default()
    }
}

impl PublisherRestriction {
    #[allow(dead_code)]
    fn new() -> Self {
        Self::default()
    }
}
