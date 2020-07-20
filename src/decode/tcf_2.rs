use crate::decode::{
    error::TcsError,
    model::{
        PublisherRestriction, PublisherRestrictionType, PublisherTC, RangeSection,
        RangeSectionType, TCModelV2, TCSegment,
    },
    util::{
        parse_from_bytes, parse_string_from_bytes, parse_u16_bitfield_from_bytes,
        parse_u8_bitfield_from_bytes, parse_vendor_range_from_bytes,
    },
};
use std::convert::TryFrom;

fn parse_publisher_restrictions_from_bytes(
    val: &[u8],
    bit_start: usize,
) -> Result<RangeSection, TcsError> {
    byte_list_bit_boundary_check!(val, bit_start + 11);

    let restriction_count = parse_from_bytes(val, bit_start, 12) as usize;
    let mut publisher_restrictions: Vec<PublisherRestriction> =
        Vec::with_capacity(restriction_count);
    let mut index: usize = 0;
    let mut bit_index = bit_start + 12;

    while index < restriction_count {
        byte_list_bit_boundary_check!(val, bit_index + 8);

        let purpose_id = parse_from_bytes(val, bit_index, 6) as u8;
        let restriction_type = parse_from_bytes(val, bit_index + 6, 2) as u8;
        let section = parse_vendor_range_from_bytes(val, bit_index + 8)?;

        bit_index = section.last_bit + 9;

        publisher_restrictions.push(PublisherRestriction {
            purpose_id,
            restriction_type: match restriction_type {
                0 => PublisherRestrictionType::NotAllowed,
                1 => PublisherRestrictionType::RequireConsent,
                2 => PublisherRestrictionType::RequireLegitimateInterest,
                _ => PublisherRestrictionType::Undefined,
            },
            vendor_list: if let RangeSectionType::Vendor(vendor_set) = section.value {
                vendor_set
            } else {
                return Err(TcsError::InvalidSectionDefinition);
            },
        });

        index += 1;
    }

    Ok(RangeSection {
        last_bit: bit_index,
        value: RangeSectionType::PublisherRestriction(publisher_restrictions),
    })
}

fn parse_range_sections_from_bytes(
    val: &[u8],
    bit_start: usize,
) -> Result<Vec<RangeSection>, TcsError> {
    let max_bit_length = val.len() * 8;
    let mut sections: Vec<RangeSection> = Vec::with_capacity(3);
    let mut start = bit_start;
    let mut section_index = 0;

    while start < max_bit_length && section_index < 3 {
        let section =
            if section_index < 2 {
                let max_vendor_id = parse_from_bytes(val, start, 16) as usize;

                if parse_from_bytes(val, start + 16, 1) == 0 {
                    RangeSection {
                        last_bit: start + 16 + max_vendor_id,
                        value: if section_index == 0 {
                            RangeSectionType::Vendor(parse_u16_bitfield_from_bytes(
                                val,
                                start + 17,
                                max_vendor_id,
                            )?)
                        } else {
                            RangeSectionType::VendorLegitimateInterest(
                                parse_u16_bitfield_from_bytes(val, start + 17, max_vendor_id)?,
                            )
                        },
                    }
                } else {
                    parse_vendor_range_from_bytes(val, start + 17)?
                }
            } else {
                parse_publisher_restrictions_from_bytes(val, start)?
            };

        start = section.last_bit + 1;
        section_index += 1;
        sections.push(section);
    }

    Ok(sections)
}

fn parse_vendor_segment_from_bytes(val: &[u8], bit_start: usize) -> Result<Vec<u16>, TcsError> {
    let max_vendor_id = parse_from_bytes(val, bit_start, 16) as usize;

    Ok(if parse_from_bytes(val, bit_start + 16, 1) == 0 {
        parse_u16_bitfield_from_bytes(val, bit_start + 17, max_vendor_id)?
    } else if let RangeSectionType::Vendor(vendor_set) =
        parse_vendor_range_from_bytes(val, bit_start + 17)?.value
    {
        vendor_set
    } else {
        return Err(TcsError::UnexpectedRangeSection);
    })
}

fn parse_publisher_tc_from_bytes(val: &[u8], bit_start: usize) -> Result<PublisherTC, TcsError> {
    let custom_purposes_count = parse_from_bytes(val, bit_start + 48, 6) as usize;

    Ok(PublisherTC {
        publisher_purposes_consent: parse_u8_bitfield_from_bytes(val, bit_start, 24)?,
        publisher_purposes_li_transparency: parse_u8_bitfield_from_bytes(val, bit_start + 24, 24)?,
        custom_purposes_consent: if custom_purposes_count > 0 {
            parse_u8_bitfield_from_bytes(val, bit_start + 54, custom_purposes_count)?
        } else {
            vec![]
        },
        custom_purposes_li_transparency: if custom_purposes_count > 0 {
            parse_u8_bitfield_from_bytes(
                val,
                bit_start + 54 + custom_purposes_count,
                custom_purposes_count,
            )?
        } else {
            vec![]
        },
    })
}

fn parse_tc_segments_from_slice(val: &[Vec<u8>]) -> Result<TCSegment, TcsError> {
    let mut tc_segment = TCSegment {
        disclosed_vendors: None,
        allowed_vendors: None,
        publisher_tc: None,
    };

    for segment in val {
        let segment_bytes = segment.as_slice();

        match parse_from_bytes(segment_bytes, 0, 3) {
            1 => {
                tc_segment.disclosed_vendors =
                    Some(parse_vendor_segment_from_bytes(segment_bytes, 3)?)
            }
            2 => {
                tc_segment.allowed_vendors =
                    Some(parse_vendor_segment_from_bytes(segment_bytes, 3)?)
            }
            3 => tc_segment.publisher_tc = Some(parse_publisher_tc_from_bytes(segment_bytes, 3)?),
            _ => return Err(TcsError::InvalidSectionDefinition),
        };
    }

    Ok(tc_segment)
}

impl TryFrom<&str> for TCModelV2 {
    type Error = TcsError;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        if !val.starts_with('C') {
            return Err(TcsError::UnsupportedVersion);
        }

        let mut tcs_segments: Vec<Vec<u8>> = Vec::with_capacity(4);

        for base64_str in val.split('.') {
            if base64_str.is_empty() {
                return Err(TcsError::InsufficientLength);
            }

            tcs_segments.push(
                match base64::decode_config(base64_str, base64::URL_SAFE_NO_PAD) {
                    Ok(decoded_bytes) => decoded_bytes,
                    Err(err) => return Err(TcsError::InvalidUrlSafeBase64(err)),
                },
            );
        }

        Self::try_from_vec(tcs_segments)
    }
}

impl TCModelV2 {
    fn try_from_vec(val: Vec<Vec<u8>>) -> Result<Self, TcsError> {
        let core_segment = val[0].as_slice();

        byte_list_bit_boundary_check!(core_segment, 213);

        let mut core_sections = parse_range_sections_from_bytes(core_segment, 213)?;
        let segments = parse_tc_segments_from_slice(&val[1..])?;
        let publisher_segment = segments.publisher_tc.unwrap_or_default();

        Ok(Self {
            created_at: parse_from_bytes(core_segment, 6, 36) * 100,
            updated_at: parse_from_bytes(core_segment, 42, 36) * 100,
            cmp_id: parse_from_bytes(core_segment, 78, 12) as u16,
            cmp_version: parse_from_bytes(core_segment, 90, 12) as u16,
            consent_screen: parse_from_bytes(core_segment, 102, 6) as u8,
            consent_language: parse_string_from_bytes(core_segment, 108, 6, 2)?,
            vendor_list_version: parse_from_bytes(core_segment, 120, 12) as u16,
            tcf_policy_version: parse_from_bytes(core_segment, 132, 6) as u16,
            is_service_specific: parse_from_bytes(core_segment, 138, 1) == 1,
            use_non_standard_stacks: parse_from_bytes(core_segment, 139, 1) == 1,
            special_feature_opt_ins: parse_u8_bitfield_from_bytes(core_segment, 140, 12)?,
            purpose_consents: parse_u8_bitfield_from_bytes(core_segment, 152, 24)?,
            purpose_li_transparency: parse_u8_bitfield_from_bytes(core_segment, 176, 24)?,
            purpose_one_treatment: parse_from_bytes(core_segment, 200, 1) == 1,
            publisher_country_code: parse_string_from_bytes(core_segment, 201, 6, 2)?,
            vendor_consents: range_section_value!(core_sections, RangeSectionType::Vendor),
            vendor_li_consents: range_section_value!(
                core_sections,
                RangeSectionType::VendorLegitimateInterest
            ),
            publisher_restrictions: range_section_value!(
                core_sections,
                RangeSectionType::PublisherRestriction
            ),
            disclosed_vendors: segments.disclosed_vendors.unwrap_or_default(),
            allowed_vendors: segments.allowed_vendors.unwrap_or_default(),
            publisher_purposes_consent: publisher_segment.publisher_purposes_consent,
            publisher_purposes_li_transparency: publisher_segment
                .publisher_purposes_li_transparency,
            custom_purposes_consent: publisher_segment.custom_purposes_consent,
            custom_purposes_li_transparency: publisher_segment.custom_purposes_li_transparency,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iab_tcf_v2_core_vendor_range() {
        assert_eq!(
            TCModelV2::try_from("COvFyGBOvFyGBAbAAAENAPCAAOAAAAAAAAAAAEEUACCKAAA"),
            Ok(TCModelV2 {
                created_at: 1582243059300,
                updated_at: 1582243059300,
                cmp_id: 27,
                cmp_version: 0,
                consent_screen: 0,
                consent_language: String::from("EN"),
                vendor_list_version: 15,
                tcf_policy_version: 2,
                is_service_specific: false,
                use_non_standard_stacks: false,
                special_feature_opt_ins: vec![],
                purpose_consents: vec![1, 2, 3],
                purpose_li_transparency: vec![],
                purpose_one_treatment: false,
                publisher_country_code: String::from("AA"),
                vendor_consents: vec![2, 6, 8],
                vendor_li_consents: vec![2, 6, 8],
                publisher_restrictions: vec![],
                disclosed_vendors: vec![],
                allowed_vendors: vec![],
                publisher_purposes_consent: vec![],
                publisher_purposes_li_transparency: vec![],
                custom_purposes_consent: vec![],
                custom_purposes_li_transparency: vec![],
            })
        );
    }

    #[test]
    fn iab_tcf_v2_core_disclosed_allowed_vendors_publisher_meta() {
        assert_eq!(
            TCModelV2::try_from("COw4XqLOw4XqLAAAAAENAXCAAAAAAAAAAAAAAAAAAAAA.IFukWSQgAIQwgI0QEByFAAAAeIAACAIgSAAQAIAgEQACEABAAAgAQFAEAIAAAGBAAgAAAAQAIFAAMCQAAgAAQiRAEQAAAAANAAIAAggAIYQFAAARmggBC3ZCYzU2yIA.QFukWSQgAIQwgI0QEByFAAAAeIAACAIgSAAQAIAgEQACEABAAAgAQFAEAIAAAGBAAgAAAAQAIFAAMCQAAgAAQiRAEQAAAAANAAIAAggAIYQFAAARmggBC3ZCYzU2yIA.YAAAAAAAAAAAAAAAAAA"),
            Ok(TCModelV2 {
                created_at: 1585246887500,
                updated_at: 1585246887500,
                cmp_id: 0,
                cmp_version: 0,
                consent_screen: 0,
                consent_language: String::from("EN"),
                vendor_list_version: 23,
                tcf_policy_version: 2,
                is_service_specific: false,
                use_non_standard_stacks: false,
                special_feature_opt_ins: vec![],
                purpose_consents: vec![],
                purpose_li_transparency: vec![],
                purpose_one_treatment: false,
                publisher_country_code: String::from("AA"),
                vendor_consents: vec![],
                vendor_li_consents: vec![],
                publisher_restrictions: vec![],
                disclosed_vendors: vec![2, 6, 8, 9, 12, 15, 18, 23, 37, 42, 47, 48, 53, 61, 65, 66, 68, 72, 80, 88, 89, 90, 93, 98, 100, 126, 127, 128, 129, 133, 153, 163, 167, 174, 177, 192, 205, 215, 224, 228, 243, 248, 262, 281, 294, 302, 304, 314, 325, 350, 351, 358, 371, 402, 415, 422, 424, 439, 440, 447, 450, 467, 486, 491, 495, 498, 502, 512, 516, 553, 554, 556, 571, 587, 593, 607, 612, 613, 618, 626, 628, 648, 652, 653, 656, 657, 659, 665, 676, 681, 683, 684, 686, 687, 688, 690, 691, 694, 699, 702, 703, 707, 708, 711, 712, 714, 716, 719, 720, 722, 723, 725, 726, 729, 733],
                allowed_vendors: vec![2, 6, 8, 9, 12, 15, 18, 23, 37, 42, 47, 48, 53, 61, 65, 66, 68, 72, 80, 88, 89, 90, 93, 98, 100, 126, 127, 128, 129, 133, 153, 163, 167, 174, 177, 192, 205, 215, 224, 228, 243, 248, 262, 281, 294, 302, 304, 314, 325, 350, 351, 358, 371, 402, 415, 422, 424, 439, 440, 447, 450, 467, 486, 491, 495, 498, 502, 512, 516, 553, 554, 556, 571, 587, 593, 607, 612, 613, 618, 626, 628, 648, 652, 653, 656, 657, 659, 665, 676, 681, 683, 684, 686, 687, 688, 690, 691, 694, 699, 702, 703, 707, 708, 711, 712, 714, 716, 719, 720, 722, 723, 725, 726, 729, 733],
                publisher_purposes_consent: vec![],
                publisher_purposes_li_transparency: vec![],
                custom_purposes_consent: vec![],
                custom_purposes_li_transparency: vec![],
            })
        );
    }

    #[test]
    fn iab_tcf_v2_core_disclosed_allowed_vendors_publisher_meta_order() {
        assert_eq!(
            TCModelV2::try_from("COw4XqLOw4XqLAAAAAENAXCAAAAAAAAAAAAAAAAAAAAA.YAAAAAAAAAAAAAAAAAA.QFukWSQgAIQwgI0QEByFAAAAeIAACAIgSAAQAIAgEQACEABAAAgAQFAEAIAAAGBAAgAAAAQAIFAAMCQAAgAAQiRAEQAAAAANAAIAAggAIYQFAAARmggBC3ZCYzU2yIA.IFukWSQgAIQwgI0QEByFAAAAeIAACAIgSAAQAIAgEQACEABAAAgAQFAEAIAAAGBAAgAAAAQAIFAAMCQAAgAAQiRAEQAAAAANAAIAAggAIYQFAAARmggBC3ZCYzU2yIA"),
            Ok(TCModelV2 {
                created_at: 1585246887500,
                updated_at: 1585246887500,
                cmp_id: 0,
                cmp_version: 0,
                consent_screen: 0,
                consent_language: String::from("EN"),
                vendor_list_version: 23,
                tcf_policy_version: 2,
                is_service_specific: false,
                use_non_standard_stacks: false,
                special_feature_opt_ins: vec![],
                purpose_consents: vec![],
                purpose_li_transparency: vec![],
                purpose_one_treatment: false,
                publisher_country_code: String::from("AA"),
                vendor_consents: vec![],
                vendor_li_consents: vec![],
                publisher_restrictions: vec![],
                disclosed_vendors: vec![2, 6, 8, 9, 12, 15, 18, 23, 37, 42, 47, 48, 53, 61, 65, 66, 68, 72, 80, 88, 89, 90, 93, 98, 100, 126, 127, 128, 129, 133, 153, 163, 167, 174, 177, 192, 205, 215, 224, 228, 243, 248, 262, 281, 294, 302, 304, 314, 325, 350, 351, 358, 371, 402, 415, 422, 424, 439, 440, 447, 450, 467, 486, 491, 495, 498, 502, 512, 516, 553, 554, 556, 571, 587, 593, 607, 612, 613, 618, 626, 628, 648, 652, 653, 656, 657, 659, 665, 676, 681, 683, 684, 686, 687, 688, 690, 691, 694, 699, 702, 703, 707, 708, 711, 712, 714, 716, 719, 720, 722, 723, 725, 726, 729, 733],
                allowed_vendors: vec![2, 6, 8, 9, 12, 15, 18, 23, 37, 42, 47, 48, 53, 61, 65, 66, 68, 72, 80, 88, 89, 90, 93, 98, 100, 126, 127, 128, 129, 133, 153, 163, 167, 174, 177, 192, 205, 215, 224, 228, 243, 248, 262, 281, 294, 302, 304, 314, 325, 350, 351, 358, 371, 402, 415, 422, 424, 439, 440, 447, 450, 467, 486, 491, 495, 498, 502, 512, 516, 553, 554, 556, 571, 587, 593, 607, 612, 613, 618, 626, 628, 648, 652, 653, 656, 657, 659, 665, 676, 681, 683, 684, 686, 687, 688, 690, 691, 694, 699, 702, 703, 707, 708, 711, 712, 714, 716, 719, 720, 722, 723, 725, 726, 729, 733],
                publisher_purposes_consent: vec![],
                publisher_purposes_li_transparency: vec![],
                custom_purposes_consent: vec![],
                custom_purposes_li_transparency: vec![],
            })
        );
    }

    #[test]
    fn iab_tcf_v2_core_vendor_vendor_li_range_1() {
        assert_eq!(
            TCModelV2::try_from("CGL23UdMFJzvuA9ACCENAXCEAC0AAGrAAA5YA5ht7-_d_7_vd-f-nrf4_4A4hM4JCKoK4YhmAqABgAEgAA"),
            Ok(TCModelV2 {
                created_at: 664138268500,
                updated_at: 1297135921400,
                cmp_id: 61,
                cmp_version: 2,
                consent_screen: 2,
                consent_language: String::from("EN"),
                vendor_list_version: 23,
                tcf_policy_version: 2,
                is_service_specific: false,
                use_non_standard_stacks: false,
                special_feature_opt_ins: vec![2],
                purpose_consents: vec![3, 5, 6, 8],
                purpose_li_transparency: vec![2, 3, 5, 7, 9, 10],
                purpose_one_treatment: false,
                publisher_country_code: String::from("HL"),
                vendor_consents: vec![4, 5, 7, 8, 10, 11, 12, 13, 15, 16, 17, 18, 19, 20, 21, 23, 24, 25, 26, 27, 28, 30, 31, 32, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 45, 46, 47, 48, 49, 50, 51, 52, 53, 55, 56, 57, 58, 60, 61, 62, 64, 65, 66, 67, 68, 69, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 83, 86, 87, 88, 89, 91, 93, 94, 96, 97, 98, 99, 100, 101, 102, 103, 107, 108, 109, 110, 111, 112, 113, 114, 115],
                vendor_li_consents: vec![4, 7, 8, 11, 12, 13, 19, 22, 27, 31, 33, 35, 37, 43, 45, 47, 48, 49, 54, 55, 59, 64, 65, 68, 69, 77, 79, 81, 94, 95, 110, 113],
                publisher_restrictions: vec![],
                disclosed_vendors: vec![],
                allowed_vendors: vec![],
                publisher_purposes_consent: vec![],
                publisher_purposes_li_transparency: vec![],
                custom_purposes_consent: vec![],
                custom_purposes_li_transparency: vec![],
            })
        );
    }

    #[test]
    fn iab_tcf_v2_core_disclosed_vendors_range() {
        assert_eq!(
            TCModelV2::try_from("COvFyGBOvFyGBAbAAAENAPCAAOAAAAAAAAAAAEEUACCKAAA.IFoEUQQgAIQwgIwQABAEAAAAOIAACAIAAAAQAIAgEAACEAAAAAgAQBAAAAAAAGBAAgAAAAAAAFAAECAAAgAAQARAEQAAAAAJAAIAAgAAAYQEAAAQmAgBC3ZAYzUw"),
            Ok(TCModelV2 {
                created_at: 1582243059300,
                updated_at: 1582243059300,
                cmp_id: 27,
                cmp_version: 0,
                consent_screen: 0,
                consent_language: String::from("EN"),
                vendor_list_version: 15,
                tcf_policy_version: 2,
                is_service_specific: false,
                use_non_standard_stacks: false,
                special_feature_opt_ins: vec![],
                purpose_consents: vec![1, 2, 3],
                purpose_li_transparency: vec![],
                purpose_one_treatment: false,
                publisher_country_code: String::from("AA"),
                vendor_consents: vec![2, 6, 8],
                vendor_li_consents: vec![2, 6, 8],
                publisher_restrictions: vec![],
                disclosed_vendors: vec![2, 6, 8, 12, 18, 23, 37, 42, 47, 48, 53, 61, 65, 66, 72, 88, 98, 127, 128, 129, 133, 153, 163, 192, 205, 215, 224, 243, 248, 281, 294, 304, 350, 351, 358, 371, 422, 424, 440, 447, 467, 486, 498, 502, 512, 516, 553, 556, 571, 587, 612, 613, 618, 626, 648, 653, 656, 657, 665, 676, 681, 683, 684, 686, 687, 688, 690, 691, 694, 702, 703, 707, 708, 711, 712, 714, 716, 719, 720],
                allowed_vendors: vec![],
                publisher_purposes_consent: vec![],
                publisher_purposes_li_transparency: vec![],
                custom_purposes_consent: vec![],
                custom_purposes_li_transparency: vec![],
            })
        );
    }

    #[test]
    fn iab_tcf_v2_core_publisher_meta() {
        assert_eq!(
            TCModelV2::try_from(
                "COw4XqLOw4XqLAAAAAENAXCAAP-gAAAfwIAAACngAI8AAA.cAEAPAAAC7gAHw4AAA"
            ),
            Ok(TCModelV2 {
                created_at: 1585246887500,
                updated_at: 1585246887500,
                cmp_id: 0,
                cmp_version: 0,
                consent_screen: 0,
                consent_language: String::from("EN"),
                vendor_list_version: 23,
                tcf_policy_version: 2,
                is_service_specific: false,
                use_non_standard_stacks: false,
                special_feature_opt_ins: vec![],
                purpose_consents: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 11],
                purpose_li_transparency: vec![12, 13, 14, 15, 16, 17, 18],
                purpose_one_treatment: true,
                publisher_country_code: String::from("AA"),
                vendor_consents: vec![2, 3, 4, 5],
                vendor_li_consents: vec![1, 2, 3, 4],
                publisher_restrictions: vec![],
                disclosed_vendors: vec![],
                allowed_vendors: vec![],
                publisher_purposes_consent: vec![1, 13, 24],
                publisher_purposes_li_transparency: vec![1, 2, 3],
                custom_purposes_consent: vec![2, 3, 4, 19, 20, 21, 22, 23],
                custom_purposes_li_transparency: vec![5, 6, 7],
            })
        );
    }

    #[test]
    fn iab_tcf_v2_core_disclosed_allowed_vendor_publisher_meta() {
        assert_eq!(
            TCModelV2::try_from("COw4XqLOw4XqLAAAAAENAXCf-v-gAAAfwIAAACngAI8AEFABgACAA4A.IAPPwAPrwA.QAPPwAPrwA.cAEAPAAAC7gAHw4AAA"),
            Ok(TCModelV2{
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
            })
        );
    }
}
