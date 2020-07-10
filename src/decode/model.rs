#[cfg_attr(test, derive(Debug))]
#[derive(PartialEq)]
pub enum TCModel {
    V1(Box<TCModelV1>),
    V2(Box<TCModelV2>),
}

#[cfg_attr(test, derive(Debug))]
#[derive(PartialEq)]
pub enum PublisherRestrictionType {
    NotAllowed,
    RequireConsent,
    RequireLegitimateInterest,
    Undefined,
}

#[cfg_attr(test, derive(Debug))]
#[derive(PartialEq)]
pub struct VendorSet {
    pub is_blocklist: bool,
    pub list: Vec<u16>,
}

#[cfg_attr(test, derive(Debug))]
#[derive(PartialEq)]
pub struct TCModelV1 {
    pub created_at: u64,
    pub updated_at: u64,
    pub cmp_id: u16,
    pub cmp_version: u16,
    pub consent_screen: u8,
    pub consent_lang: String,
    pub vendor_list_version: u16,
    pub purpose_consents: Vec<u8>,
    pub vendors: VendorSet,
}

#[cfg_attr(test, derive(Debug))]
#[derive(PartialEq)]
pub struct TCModelV2 {
    pub created_at: u64,
    pub updated_at: u64,
    pub cmp_id: u16,
    pub cmp_version: u16,
    pub consent_screen: u8,
    pub consent_language: String,
    pub vendor_list_version: u16,
    pub tcf_policy_version: u16,
    pub is_service_specific: bool,
    pub use_non_standard_stacks: bool,
    pub special_feature_opt_ins: Vec<u8>,
    pub purpose_consents: Vec<u8>,
    pub purpose_li_transparency: Vec<u8>,
    pub purpose_one_treatment: bool,
    pub publisher_country_code: String,
    pub vendor_consents: Vec<u16>,
    pub vendor_li_consents: Vec<u16>,
    pub publisher_restrictions: Vec<PublisherRestriction>,
    pub disclosed_vendors: Vec<u16>,
    pub allowed_vendors: Vec<u16>,
    pub publisher_purposes_consent: Vec<u8>,
    pub publisher_purposes_li_transparency: Vec<u8>,
    pub custom_purposes_consent: Vec<u8>,
    pub custom_purposes_li_transparency: Vec<u8>,
}

#[cfg_attr(test, derive(Debug))]
#[derive(PartialEq)]
pub struct PublisherRestriction {
    pub purpose_id: u8,
    pub restriction_type: PublisherRestrictionType,
    pub vendor_list: Vec<u16>,
}

pub type TCSDecodeError = &'static str;

#[cfg_attr(test, derive(Debug))]
#[derive(PartialEq)]
pub(crate) enum RangeSectionType {
    Vendor(Vec<u16>),
    VendorLegitimateInterest(Vec<u16>),
    PublisherRestriction(Vec<PublisherRestriction>),
}

#[cfg_attr(test, derive(Debug))]
pub(crate) struct TCSegment {
    pub disclosed_vendors: Option<Vec<u16>>,
    pub allowed_vendors: Option<Vec<u16>>,
    pub publisher_tc: Option<PublisherTC>,
}

#[cfg_attr(test, derive(Debug))]
pub(crate) struct RangeSection {
    pub last_bit: usize,
    pub value: RangeSectionType,
}

#[cfg_attr(test, derive(Debug))]
#[derive(PartialEq, PartialOrd)]
pub(crate) struct PublisherTC {
    pub publisher_purposes_consent: Vec<u8>,
    pub publisher_purposes_li_transparency: Vec<u8>,
    pub custom_purposes_consent: Vec<u8>,
    pub custom_purposes_li_transparency: Vec<u8>,
}

impl Default for PublisherTC {
    fn default() -> Self {
        Self {
            custom_purposes_consent: vec![],
            custom_purposes_li_transparency: vec![],
            publisher_purposes_consent: vec![],
            publisher_purposes_li_transparency: vec![],
        }
    }
}
