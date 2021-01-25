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

const VENDOR_RANGE_SECTION_TYPES: &[fn(std::vec::Vec<u16>) -> RangeSectionType; 2] = &[
    RangeSectionType::Vendor,
    RangeSectionType::VendorLegitimateInterest,
];

fn parse_publisher_restrictions_from_bytes(
    val: &[u8],
    bit_start: usize,
) -> Result<RangeSection, TcsError> {
    byte_list_bit_boundary_check!(val, bit_start + 12);

    let restriction_count = parse_from_bytes(val, bit_start, 12) as usize;
    let mut publisher_restrictions: Vec<PublisherRestriction> =
        Vec::with_capacity(restriction_count);
    let mut index: usize = 0;
    let mut bit_index = bit_start + 12;

    while index < restriction_count {
        byte_list_bit_boundary_check!(val, bit_index + 8);

        let purpose_id = parse_from_bytes(val, bit_index, 6) as u8;
        let restriction_type = parse_from_bytes(val, bit_index + 6, 2) as u8;
        let section = parse_vendor_range_from_bytes(val, bit_index + 8, &RangeSectionType::Vendor)?;

        bit_index = section.last_bit;

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
        let section = if section_index < 2 {
            if parse_from_bytes(val, start + 16, 1) == 0 {
                let max_vendor_id = parse_from_bytes(val, start, 16) as usize;
                let bitfield_value = parse_u16_bitfield_from_bytes(val, start + 17, max_vendor_id)?;

                RangeSection {
                    last_bit: start + 17 + max_vendor_id,
                    value: VENDOR_RANGE_SECTION_TYPES[section_index](bitfield_value),
                }
            } else {
                parse_vendor_range_from_bytes(
                    val,
                    start + 17,
                    &VENDOR_RANGE_SECTION_TYPES[section_index],
                )?
            }
        } else {
            parse_publisher_restrictions_from_bytes(val, start)?
        };

        start = section.last_bit;
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
        parse_vendor_range_from_bytes(val, bit_start + 17, &RangeSectionType::Vendor)?.value
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
            _ => return Err(TcsError::InvalidSegmentDefinition),
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
            purposes_consent: parse_u8_bitfield_from_bytes(core_segment, 152, 24)?,
            purposes_li_transparency: parse_u8_bitfield_from_bytes(core_segment, 176, 24)?,
            purpose_one_treatment: parse_from_bytes(core_segment, 200, 1) == 1,
            publisher_country_code: parse_string_from_bytes(core_segment, 201, 6, 2)?,
            vendors_consent: range_section_value!(core_sections, RangeSectionType::Vendor),
            vendors_li_consent: range_section_value!(
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
                purposes_consent: vec![1, 2, 3],
                purposes_li_transparency: vec![],
                purpose_one_treatment: false,
                publisher_country_code: String::from("AA"),
                vendors_consent: vec![2, 6, 8],
                vendors_li_consent: vec![2, 6, 8],
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
                purposes_consent: vec![],
                purposes_li_transparency: vec![],
                purpose_one_treatment: false,
                publisher_country_code: String::from("AA"),
                vendors_consent: vec![],
                vendors_li_consent: vec![],
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
                purposes_consent: vec![],
                purposes_li_transparency: vec![],
                purpose_one_treatment: false,
                publisher_country_code: String::from("AA"),
                vendors_consent: vec![],
                vendors_li_consent: vec![],
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
                purposes_consent: vec![3, 5, 6, 8],
                purposes_li_transparency: vec![2, 3, 5, 7, 9, 10],
                purpose_one_treatment: false,
                publisher_country_code: String::from("HL"),
                vendors_consent: vec![4, 5, 7, 8, 10, 11, 12, 13, 15, 16, 17, 18, 19, 20, 21, 23, 24, 25, 26, 27, 28, 30, 31, 32, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 45, 46, 47, 48, 49, 50, 51, 52, 53, 55, 56, 57, 58, 60, 61, 62, 64, 65, 66, 67, 68, 69, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 83, 86, 87, 88, 89, 91, 93, 94, 96, 97, 98, 99, 100, 101, 102, 103, 107, 108, 109, 110, 111, 112, 113, 114, 115],
                vendors_li_consent: vec![4, 7, 8, 11, 12, 13, 19, 22, 27, 31, 33, 35, 37, 43, 45, 47, 48, 49, 54, 55, 59, 64, 65, 68, 69, 77, 79, 81, 94, 95, 110, 113],
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
                purposes_consent: vec![1, 2, 3],
                purposes_li_transparency: vec![],
                purpose_one_treatment: false,
                publisher_country_code: String::from("AA"),
                vendors_consent: vec![2, 6, 8],
                vendors_li_consent: vec![2, 6, 8],
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
                purposes_consent: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 11],
                purposes_li_transparency: vec![12, 13, 14, 15, 16, 17, 18],
                purpose_one_treatment: true,
                publisher_country_code: String::from("AA"),
                vendors_consent: vec![2, 3, 4, 5],
                vendors_li_consent: vec![1, 2, 3, 4],
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
                purposes_consent: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 11],
                purposes_li_transparency: vec![12, 13, 14, 15, 16, 17, 18],
                purpose_one_treatment: true,
                publisher_country_code: String::from("AA"),
                vendors_consent: vec![2, 3, 4, 5],
                vendors_li_consent: vec![1, 2, 3, 4],
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

    #[test]
    fn iab_tcf_v2_core_publisher_meta_2() {
        assert_eq!(
            TCModelV2::try_from("CO51ctPO51ctPCnABBDEA3CsAP_AAAAAAAYgGkNf_X_fb2vj-_5999t0eY1f9_63v-wzjgeNs-8Nyd_X_L4Xr2MyvB36pq4KuR4Eu3LBAQdlHOHcTQmQwIkVqTLsbk2Mq7NKJ7LEilMbM2dYGH9vn9XTuZCY70_sf__z_3-_-___67f-L2wAAADhIBQAFQAQAA0ACYAE8ARwAtwB-gIvAXmKgBgBMAEcAvMZADACYAI4BeY6AaABUAEAANAAmABPAEcAJgAW4A_QCLAIvAXmAxglAFACYAI4AW4CLwF5lIBYAFQAQAA0ACYAE8AW4A_QCLAIvAXmAxghACACYAI4.f_gAAAAAAWAA"),
            Ok(
                TCModelV2 {
                    created_at: 1600269806300,
                    updated_at: 1600269806300,
                    cmp_id: 167,
                    cmp_version: 1,
                    consent_screen: 1,
                    consent_language: String::from("DE"),
                    vendor_list_version: 55,
                    tcf_policy_version: 2,
                    is_service_specific: true,
                    use_non_standard_stacks: false,
                    special_feature_opt_ins: vec![1,2],
                    purposes_consent: (1..11).collect(),
                    purposes_li_transparency: vec![],
                    purpose_one_treatment: false,
                    publisher_country_code: String::from("DE"),
                    vendors_consent: vec![1,2,4,6,7,8,9,10,11,12,13,14,15,16,18,20,21,22,23,24,25,26,27,28,30,31,32,33,34,36,37,39,40,41,42,44,45,47,49,50,51,52,53,57,58,59,60,61,62,63,65,66,67,68,69,70,71,72,73,76,77,78,79,80,82,83,84,85,86,88,89,90,91,92,94,95,97,98,100,101,102,104,108,109,110,111,114,115,119,120,122,124,126,127,128,129,130,131,132,133,134,136,137,138,139,140,141,142,143,144,145,147,149,150,152,153,154,155,157,158,159,160,161,162,163,164,165,167,168,173,174,177,178,179,183,184,185,192,193,194,195,199,200,202,203,205,206,209,210,211,212,213,215,216,217,218,223,224,226,227,228,231,234,235,236,238,239,240,241,242,243,244,246,248,249,250,251,252,253,254,255,256,259,261,262,263,264,265,270,272,273,274,275,277,279,280,281,282,284,285,289,290,293,294,297,299,301,302,303,304,310,311,312,314,315,316,317,318,319,321,323,325,328,329,331,333,335,336,337,343,345,347,349,350,351,354,358,359,360,361,368,371,373,374,375,377,378,380,381,382,385,387,388,394,402,408,409,410,412,413,416,418,422,423,424,427,428,429,434,435,436,438,439,440,444,447,448,450,455,458,459,462,467,468,475,479,482,486,488,490,491,493,495,498,501,502,505,507,508,509,511,512,516,517,519,520,521,524,527,528,530,531,535,536,539,541,543,545,546,547,549,550,553,554,556,559,561,565,568,569,570,571,573,574,577,579,580,584,587,591,593,596,598,601,602,606,607,609,610,613,614,617,618,620,621,624,625,626,628,630,631,638,639,644,645,646,647,648,649,650,652,653,655,656,657,658,659,662,663,664,665,666,667,668,670,672,674,675,676,678,681,682,683,685,686,687,690,691,694,699,702,703,707,708,709,711,712,713,714,716,719,720,721,722,723,724,725,727,728,732,733,734,735,736,737,738,739,740,741,742,743,744,745,746,747,748,749,750,753,754,755,756,757,758,759,760,761,762,764,765,766,767,768,769,770,771,773,774,775,776,777,778,779,780,781,782,783,785,786,787,788,789,790,791,792,793,794,795,796,797,798,799,800,801,802,803,804,805,807,809,810,811,813,814,816,817,818,819,820,821,822,823,824,825,829,831,832,833,834,836,837,839,840],
                    vendors_li_consent: vec![],
                    publisher_restrictions: vec![
                        PublisherRestriction {
                            purpose_id: 2,
                            restriction_type: PublisherRestrictionType::RequireConsent,
                            vendor_list: vec![21, 32, 52, 76, 79, 142, 183, 253, 559, 755],
                        },
                        PublisherRestriction {
                            purpose_id: 5,
                            restriction_type: PublisherRestrictionType::RequireConsent,
                            vendor_list: vec![76, 142, 755],
                        },
                        PublisherRestriction {
                            purpose_id: 6,
                            restriction_type: PublisherRestrictionType::RequireConsent,
                            vendor_list: vec![76, 142, 755],
                        },
                        PublisherRestriction {
                            purpose_id: 7,
                            restriction_type: PublisherRestrictionType::RequireConsent,
                            vendor_list: vec![21, 32, 52, 76, 79, 142, 152, 183, 253, 278, 559, 755, 792],
                        },
                        PublisherRestriction {
                            purpose_id: 9,
                            restriction_type: PublisherRestrictionType::RequireConsent,
                            vendor_list: vec![76, 142, 183, 559, 755],
                        },
                        PublisherRestriction {
                            purpose_id: 10,
                            restriction_type: PublisherRestrictionType::RequireConsent,
                            vendor_list: vec![21, 32, 52, 76, 79, 183, 253, 278, 559, 755, 792],
                        },
                        PublisherRestriction {
                            purpose_id: 8,
                            restriction_type: PublisherRestrictionType::RequireConsent,
                            vendor_list: vec![76, 142],
                        },
                    ],
                    disclosed_vendors: vec![],
                    allowed_vendors: vec![],
                    publisher_purposes_consent: (1..11).collect(),
                    publisher_purposes_li_transparency: vec![],
                    custom_purposes_consent: vec![1, 2],
                    custom_purposes_li_transparency: vec![],
                }
            )
        );
    }

    #[test]
    fn iab_tcf_v2_core_disclosed_allowed_vendor_publisher_meta_2() {
        assert_eq!(
            TCModelV2::try_from("COw4XqLOw4XqLAAAAAENAXCf-v-gAAAfwIAAACngAI8AIFABgACAA4SADAAgADQ.IAPPwAPrwA.QAPPwAPrwA.cAEAPAAAC7gAHw4AAA"),
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
                use_non_standard_stacks: true,
                special_feature_opt_ins: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 11],
                purposes_consent: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 11],
                purposes_li_transparency: vec![12, 13, 14, 15, 16, 17, 18],
                purpose_one_treatment: true,
                publisher_country_code: String::from("AA"),
                vendors_consent: vec![2, 3, 4, 5],
                vendors_li_consent: vec![1, 2, 3, 4],
                publisher_restrictions: vec![
                    PublisherRestriction {
                        purpose_id: 1,
                        restriction_type: PublisherRestrictionType::RequireConsent,
                        vendor_list: vec![1, 2, 3, 4, 5, 6, 7],
                    },
                    PublisherRestriction {
                        purpose_id: 2,
                        restriction_type: PublisherRestrictionType::RequireConsent,
                        vendor_list: vec![8, 9, 10, 11, 12, 13],
                    },
                ],
                disclosed_vendors: vec![1, 2, 3, 4, 5, 6, 19, 20, 21, 22, 23, 25, 27, 28, 29, 30],
                allowed_vendors: vec![1, 2, 3, 4, 5, 6, 19, 20, 21, 22, 23, 25, 27, 28, 29, 30],
                publisher_purposes_consent: vec![1, 13, 24],
                publisher_purposes_li_transparency: vec![1, 2, 3],
                custom_purposes_consent: vec![2, 3, 4, 19, 20, 21, 22, 23],
                custom_purposes_li_transparency: vec![5, 6, 7],
            })
        );
    }

    #[test]
    fn iab_tcf_v2_core_publisher_meta_3() {
        assert_eq!(
            TCModelV2::try_from("CO4yYChO4yYChCnABBDEA0CsAP_AAAAAAAYgF-wDwAUAB6AEaAK4AaYA5AC6gH_ARqAkEBQ4CuwFvgLsAX6AAAAYJABAXmKgAgLzGQAQF5joAIC8yUAEBeZSACAvMAAA.f_gAAAAAAQAA"),
            Ok(TCModelV2 {
                created_at: 1598511529700,
                updated_at: 1598511529700,
                cmp_id: 167,
                cmp_version: 1,
                consent_screen: 1,
                consent_language: String::from("DE"),
                vendor_list_version: 52,
                tcf_policy_version: 2,
                is_service_specific: true,
                use_non_standard_stacks: false,
                special_feature_opt_ins: vec![1, 2],
                purposes_consent: (1..11).collect(),
                purposes_li_transparency: vec![],
                purpose_one_treatment: false,
                publisher_country_code: String::from("DE"),
                vendors_consent: vec![40, 122, 141, 174, 211, 228, 373, 511, 565, 577, 647, 699, 735, 748, 765],
                vendors_li_consent: vec![],
                publisher_restrictions: vec![
                    PublisherRestriction {
                        purpose_id: 2,
                        restriction_type: PublisherRestrictionType::RequireConsent,
                        vendor_list: vec![755]
                    },
                    PublisherRestriction {
                        purpose_id: 5,
                        restriction_type: PublisherRestrictionType::RequireConsent,
                        vendor_list: vec![755]
                    },
                    PublisherRestriction {
                        purpose_id: 6,
                        restriction_type: PublisherRestrictionType::RequireConsent,
                        vendor_list: vec![755]
                    },
                    PublisherRestriction {
                        purpose_id: 7,
                        restriction_type: PublisherRestrictionType::RequireConsent,
                        vendor_list: vec![755]
                    },
                    PublisherRestriction {
                        purpose_id: 9,
                        restriction_type: PublisherRestrictionType::RequireConsent,
                        vendor_list: vec![755]
                    },
                    PublisherRestriction {
                        purpose_id: 10,
                        restriction_type: PublisherRestrictionType::RequireConsent,
                        vendor_list: vec![755]
                    }
                ],
                disclosed_vendors: vec![],
                allowed_vendors: vec![],
                publisher_purposes_consent: (1..11).collect(),
                publisher_purposes_li_transparency: vec![],
                custom_purposes_consent: vec![],
                custom_purposes_li_transparency: vec![]
            })
        );
    }

    #[test]
    fn iab_tcf2_core_section_parsing() {
        assert_eq!(
            TCModelV2::try_from("CO-Z5geO-Z5geAfbgBDEBECoAP_AAH_AAAigGfwFgADAAZABOACoAFgAMgAiAB-AERAIwAjQBMAEWAJwAXMAzgCCgEtALaAXmAxEBmgDPwM_gLAAGAAyACcAFQALAAZABEAD8AIiARgBGgCYAIsATgAuYBnAEFAJaAW0AvMBiIDNAGfgAA"),
            Ok(TCModelV2 {
                created_at: 1607936207800,
                updated_at: 1607936207800,
                cmp_id: 31,
                cmp_version: 1760,
                consent_screen: 1,
                consent_language: String::from("DE"),
                vendor_list_version: 68,
                tcf_policy_version: 2,
                is_service_specific: true,
                use_non_standard_stacks: false,
                special_feature_opt_ins: vec![1],
                purposes_consent: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10], 
                purposes_li_transparency: vec![2, 3, 4, 5, 6, 7, 8, 9, 10], 
                purpose_one_treatment: false, 
                publisher_country_code: String::from("EU"),
                vendors_consent: vec![6, 25, 39, 42, 44, 50, 68, 126, 136, 140, 141, 152, 278, 312, 371, 412, 522, 602, 730, 755, 785, 820, 831], 
                vendors_li_consent: vec![6, 25, 39, 42, 44, 50, 68, 126, 136, 140, 141, 152, 278, 312, 371, 412, 522, 602, 730, 755, 785, 820, 831],
                publisher_restrictions: vec![], 
                disclosed_vendors: vec![], 
                allowed_vendors: vec![], 
                publisher_purposes_consent: vec![],
                publisher_purposes_li_transparency: vec![], 
                custom_purposes_consent: vec![],
                custom_purposes_li_transparency: vec![]
            })
        );
    }
}
