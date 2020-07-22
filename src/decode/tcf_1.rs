use crate::decode::{
    error::TcsError,
    model::{RangeSectionType, TCModelV1, VendorSet},
    util::{
        parse_from_bytes, parse_string_from_bytes, parse_u16_bitfield_from_bytes,
        parse_u8_bitfield_from_bytes, parse_vendor_range_from_bytes,
    },
};
use std::convert::TryFrom;

impl TryFrom<&str> for TCModelV1 {
    type Error = TcsError;

    fn try_from(val: &str) -> Result<Self, TcsError> {
        match base64::decode_config(val, base64::URL_SAFE_NO_PAD) {
            Ok(decoded_bytes) => {
                byte_list_bit_boundary_check!(val, 5);

                if parse_from_bytes(&decoded_bytes, 0, 6) != 1 {
                    return Err(TcsError::UnsupportedVersion);
                }

                Self::try_from_vec(decoded_bytes)
            }
            Err(err) => Err(TcsError::InvalidUrlSafeBase64(err)),
        }
    }
}

impl TCModelV1 {
    fn try_from_vec(val: Vec<u8>) -> Result<Self, TcsError> {
        byte_list_bit_boundary_check!(val, 173);

        let max_vendor_id = parse_from_bytes(&val, 156, 16);

        Ok(Self {
            created_at: parse_from_bytes(&val, 6, 36),
            updated_at: parse_from_bytes(&val, 42, 36),
            cmp_id: parse_from_bytes(&val, 78, 12) as u16,
            cmp_version: parse_from_bytes(&val, 90, 12) as u16,
            consent_screen: parse_from_bytes(&val, 102, 6) as u8,
            consent_lang: parse_string_from_bytes(&val, 108, 6, 2)?,
            vendor_list_version: parse_from_bytes(&val, 120, 12) as u16,
            purposes_consent: parse_u8_bitfield_from_bytes(&val, 132, 24)?,
            vendors: if parse_from_bytes(&val, 172, 1) == 0 {
                VendorSet {
                    is_blocklist: false,
                    list: parse_u16_bitfield_from_bytes(&val, 173, max_vendor_id as usize)?,
                }
            } else if let RangeSectionType::Vendor(vendor_list) =
                parse_vendor_range_from_bytes(&val, 174)?.value
            {
                VendorSet {
                    is_blocklist: parse_from_bytes(&val, 173, 1) == 1,
                    list: vendor_list,
                }
            } else {
                return Err(TcsError::InvalidSectionDefinition);
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iab_tcf_v1_single_blocked_vendor() {
        assert_eq!(
            TCModelV1::try_from("BOEFEAyOEFEAyAHABDENAI4AAAB9vABAASA"),
            Ok(TCModelV1 {
                created_at: 15100821554,
                updated_at: 15100821554,
                cmp_id: 7,
                cmp_version: 1,
                consent_screen: 3,
                consent_lang: String::from("EN"),
                vendor_list_version: 8,
                purposes_consent: vec![1, 2, 3],
                vendors: VendorSet {
                    is_blocklist: true,
                    list: vec![9],
                },
            })
        );
    }

    #[test]
    fn iab_tcf_v1_allowed_vendor_range_1() {
        assert_eq!(
                TCModelV1::try_from("BOyRMJVO2IaNjAKAiBENDR-AAAAwxrv7_77e_9f-_f__9uj3Gr_v_f__3mccL5tv3hv7v6_7fi_-1nV4u_1tft9ydk1-5YtDzto507iakiPHmqNeb1n_mz1eZpRP58E09j53z7Ew_v8_v-b7BCPN_Y3v-8K96lGA"),
                Ok(TCModelV1 {
                    created_at: 15875752533,
                    updated_at: 15940559715,
                    cmp_id: 10,
                    cmp_version: 34,
                    consent_screen: 1,
                    consent_lang: String::from("EN"),
                    vendor_list_version: 209,
                    purposes_consent: vec![1, 2, 3, 4, 5],
                    vendors: VendorSet {
                        is_blocklist: false,
                        list: vec![1, 2, 4, 6, 7, 8, 10, 11, 12, 13, 14, 15, 16, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 30, 31, 32, 33, 34, 36, 37, 39, 40, 41, 42, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 55, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 68, 69, 70, 71, 72, 73, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 97, 98, 100, 101, 102, 104, 108, 109, 110, 111, 113, 114, 115, 119, 120, 122, 124, 126, 127, 128, 129, 130, 131, 132, 133, 134, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 147, 148, 149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 167, 168, 169, 170, 173, 174, 177, 178, 179, 183, 184, 185, 190, 192, 193, 194, 195, 196, 199, 200, 202, 203, 205, 206, 208, 209, 210, 211, 212, 213, 215, 216, 217, 218, 223, 224, 226, 227, 228, 229, 230, 231, 232, 234, 235, 236, 238, 239, 240, 241, 242, 243, 244, 246, 248, 249, 250, 251, 252, 253, 254, 255, 256, 258, 259, 261, 262, 263, 264, 265, 266, 270, 272, 273, 274, 275, 276, 277, 278, 279, 280, 281, 282, 284, 285, 287, 289, 290, 293, 294, 295, 297, 299, 301, 302, 303, 304, 308, 310, 311, 312, 314, 315, 316, 317, 318, 319, 320, 321, 323, 325, 326, 328, 329, 331, 333, 334, 335, 336, 337, 338, 340, 341, 343, 344, 345, 346, 347, 349, 350, 351, 354, 357, 358, 359, 361, 362, 365, 368, 369, 371, 373, 374, 375, 376, 377, 378, 380, 381, 382, 385, 387, 388, 392, 394, 395, 397, 402, 403, 404, 405, 408, 409, 410, 412, 413, 415, 416, 418, 422, 423, 424, 427, 428, 429, 431, 434, 435, 436, 438, 439, 440, 444, 447, 448, 450, 452, 455, 458, 462, 466, 467, 468, 469, 473, 474, 475, 476, 479, 480, 482, 484, 486, 490, 491, 493, 495, 496, 497, 498, 501, 502, 504, 505, 506, 507, 509, 511, 512, 515, 516, 517, 518, 519, 520, 521, 522, 523, 524, 527, 528, 530, 531, 534, 535, 536, 537, 539, 541, 543, 544, 545, 546, 549, 550, 553, 554, 556, 559, 561, 565, 568, 569, 570, 571, 572, 573, 574, 577, 578, 579, 580, 581, 587, 590, 591, 593, 596, 597, 598, 599, 601, 602, 606, 607, 608, 609, 610, 613, 614, 615, 617, 618, 619, 620, 621, 624, 625, 626, 627, 628, 630, 631, 635, 638, 639, 644, 645, 646, 647, 648, 649, 650, 652, 653, 654, 655, 656, 657, 658, 659, 662, 663, 664, 665, 666, 667, 668, 670, 671, 672, 673, 674, 675, 676, 677, 678, 681, 682, 684, 685, 686, 687, 688, 690, 691, 697, 702, 706, 707, 708, 709, 712, 713, 715, 716, 717, 718, 719, 720, 721, 723, 724, 728, 729, 731, 732, 733, 734, 736, 737, 738, 739, 740, 741, 742, 743, 744, 746, 747, 748, 749, 754, 756, 758, 759, 760, 761, 763, 764, 765, 766, 768, 770, 773, 775, 779, 780],
                    },
                })
            );
    }

    #[test]
    fn iab_tcf_v1_allowed_vendor_range_2() {
        assert_eq!(
            TCModelV1::try_from("BO2IUIWO2IUIWB9ABADEDR-AAAAwyABgACBhgA"),
            Ok(TCModelV1 {
                created_at: 15940534806,
                updated_at: 15940534806,
                cmp_id: 125,
                cmp_version: 1,
                consent_screen: 0,
                consent_lang: String::from("DE"),
                vendor_list_version: 209,
                purposes_consent: vec![1, 2, 3, 4, 5],
                vendors: VendorSet {
                    is_blocklist: false,
                    list: (1..781).collect::<Vec<u16>>(),
                },
            })
        );
    }
}
