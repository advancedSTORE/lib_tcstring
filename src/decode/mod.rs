use std::convert::TryFrom;

pub mod error;
pub mod model;
pub mod tcf_1;
pub mod tcf_2;
pub(crate) mod util;

use crate::decode::{
    error::UNSUPPORTED_VERSION,
    model::{TCModel, TCModelV1, TCModelV2, TCSDecodeError},
};

impl TryFrom<&str> for TCModel {
    type Error = TCSDecodeError;

    fn try_from(val: &str) -> Result<TCModel, Self::Error> {
        Ok(match val.chars().next() {
            Some('B') => TCModel::V1(Box::new(TCModelV1::try_from(val)?)),
            Some('C') => TCModel::V2(Box::new(TCModelV2::try_from(val)?)),
            _ => return Err(UNSUPPORTED_VERSION),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decode::model::{PublisherRestriction, PublisherRestrictionType, VendorSet};

    #[test]
    fn iab_tcf_v1_tc_model_sample() {
        assert_eq!(
            TCModel::try_from("BOEFEAyOEFEAyAHABDENAI4AAAB9vABAASA"),
            Ok(TCModel::V1(Box::new(TCModelV1 {
                created_at: 15100821554,
                updated_at: 15100821554,
                cmp_id: 7,
                cmp_version: 1,
                consent_screen: 3,
                consent_lang: String::from("EN"),
                vendor_list_version: 8,
                purpose_consents: vec![1, 2, 3],
                vendors: VendorSet {
                    is_blocklist: true,
                    list: vec![9],
                },
            })))
        );
    }

    #[test]
    fn iab_tcf_v2_tc_model_sample() {
        assert_eq!(
            TCModel::try_from("COw4XqLOw4XqLAAAAAENAXCf-v-gAAAfwIAAACngAI8AEFABgACAA4A.IAPPwAPrwA.QAPPwAPrwA.cAEAPAAAC7gAHw4AAA"),
            Ok(TCModel::V2(Box::new(TCModelV2{
                created_at: 1585246887500,
                updated_at: 1585246887500,
                cmp_id: 0,
                cmp_version: 0,
                consent_screen: 0,
                consent_language: String::from("EN"),
                vendor_list_version: 23,
                tcf_policy_version: 2,
                is_service_specific: false,
                use_non_standard_stacks: true,
                special_feature_opt_ins: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 11],
                purpose_consents: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 11],
                purpose_li_transparency: vec![12, 13, 14, 15, 16, 17, 18],
                purpose_one_treatment: true,
                publisher_country_code: String::from("AA"),
                vendor_consents: vec![2, 3, 4, 5],
                vendor_li_consents: vec![1, 2, 3, 4],
                publisher_restrictions: vec![PublisherRestriction {
                    purpose_id: 1,
                    restriction_type: PublisherRestrictionType::RequireConsent,
                    vendor_list: vec![1, 2, 3, 4, 5, 6, 7]
                }],
                disclosed_vendors: vec![1, 2, 3, 4, 5, 6, 19, 20, 21, 22, 23, 25, 27, 28, 29, 30],
                allowed_vendors: vec![1, 2, 3, 4, 5, 6, 19, 20, 21, 22, 23, 25, 27, 28, 29, 30],
                publisher_purposes_consent: vec![1, 13, 24],
                publisher_purposes_li_transparency: vec![1, 2, 3],
                custom_purposes_consent: vec![2, 3, 4, 19, 20, 21, 22, 23],
                custom_purposes_li_transparency: vec![5, 6, 7],
            })))
        );
    }
}
