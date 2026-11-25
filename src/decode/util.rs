use crate::decode::{
    error::TcsError,
    model::{RangeSection, RangeSectionType},
};

pub(crate) fn parse_from_bytes(
    val: &[u8],
    absolute_start_bit: usize,
    bit_length: usize,
) -> Result<u64, TcsError> {
    byte_list_bit_boundary_check!(val, absolute_start_bit + bit_length);

    let first_byte_start_bit = (absolute_start_bit % 8) as u8;
    let relative_end_bit = bit_length - 1;
    let absolute_end_bit = absolute_start_bit + relative_end_bit;
    let last_byte_end_bit = (absolute_end_bit % 8) as u64;
    let last_byte_index = absolute_end_bit / 8;
    let remaining_bits_in_first_byte =
        (7i64 - (first_byte_start_bit as i64 + (relative_end_bit as i64))).max(0) as u8;
    let mut bit_mask: u64 = (0xff << first_byte_start_bit) & 0xff;
    let mut current_byte = absolute_start_bit / 8;

    bit_mask = (bit_mask >> (first_byte_start_bit + remaining_bits_in_first_byte))
        << remaining_bits_in_first_byte;

    let mut return_value = (val[current_byte] as u64 & bit_mask) >> remaining_bits_in_first_byte;

    if current_byte >= last_byte_index {
        return Ok(return_value);
    }

    current_byte += 1;

    while current_byte < last_byte_index {
        return_value = (return_value << 8) | (val[current_byte] as u64);
        current_byte += 1;
    }

    let bit_shift = 7 - last_byte_end_bit;

    Ok((return_value << (last_byte_end_bit + 1))
        | ((val[current_byte] as u64 & (0xff << bit_shift)) >> bit_shift))
}

pub(crate) fn parse_string_from_bytes(
    val: &[u8],
    bit_start: usize,
    bit_width: usize,
    char_count: usize,
) -> Result<String, TcsError> {
    byte_list_bit_boundary_check!(val, bit_start + (char_count * bit_width));

    let mut result = String::with_capacity(char_count);
    let mut offset = 0;

    for _ in 0..char_count {
        let alphabet_offset = parse_from_bytes(val, bit_start + offset, bit_width)? as u8;

        if alphabet_offset > 25 {
            return Err(TcsError::InvalidAlphabetOffset);
        }

        result.push((b'A' + alphabet_offset) as char);
        offset += bit_width;
    }

    Ok(result)
}

pub(crate) fn parse_vendor_range_from_bytes(
    val: &[u8],
    bit_start: usize,
    value_type: &dyn Fn(Vec<u16>) -> RangeSectionType,
) -> Result<RangeSection, TcsError> {
    let mut bit_index = bit_start + 12;

    byte_list_bit_boundary_check!(val, bit_index);

    let num_entries = parse_from_bytes(val, bit_start, 12)? as u16;
    let mut entry_list: Vec<u16> = Vec::new();
    let mut count = 0u16;

    while count < num_entries {
        if parse_from_bytes(val, bit_index, 1)? as u8 == 1 {
            byte_list_bit_boundary_check!(val, bit_index + 33);

            let start_vendor_id = parse_from_bytes(val, bit_index + 1, 16)? as u16;
            let end_vendor_id = parse_from_bytes(val, bit_index + 17, 16)? as u16;

            for vendor_id in start_vendor_id..=end_vendor_id {
                entry_list.push(vendor_id);
            }

            bit_index += 33;
        } else {
            byte_list_bit_boundary_check!(val, bit_index + 17);

            entry_list.push(parse_from_bytes(val, bit_index + 1, 16)? as u16);
            bit_index += 17;
        }

        count += 1;
    }

    Ok(RangeSection {
        last_bit: bit_index,
        value: value_type(entry_list),
    })
}

parse_bitfield_from_bytes!(parse_u8_bitfield_from_bytes, u8);
parse_bitfield_from_bytes!(parse_u16_bitfield_from_bytes, u16);

#[cfg(test)]
mod tests {
    use super::*;

    fn vendor_list(section: RangeSection) -> Vec<u16> {
        match section.value {
            RangeSectionType::Vendor(list) => list,
            _ => panic!("expected RangeSectionType::Vendor"),
        }
    }

    #[test]
    fn parse_from_bytes_reads_up_to_last_bit() {
        assert_eq!(parse_from_bytes(&[0xff], 0, 8), Ok(0xff));
        assert_eq!(parse_from_bytes(&[0xab, 0xcd], 4, 8), Ok(0xbc));
        assert_eq!(parse_from_bytes(&[0xab, 0xcd], 15, 1), Ok(1));
    }

    #[test]
    fn parse_from_bytes_past_end_returns_err() {
        assert_eq!(
            parse_from_bytes(&[], 0, 1),
            Err(TcsError::InsufficientLength)
        );
        assert_eq!(
            parse_from_bytes(&[0xff], 1, 8),
            Err(TcsError::InsufficientLength)
        );
        assert_eq!(
            parse_from_bytes(&[0xff], 8, 1),
            Err(TcsError::InsufficientLength)
        );
        assert_eq!(
            parse_from_bytes(&[0xff, 0xff], 0, 17),
            Err(TcsError::InsufficientLength)
        );
    }

    /// NumEntries = 1 occupies bits 4..16; the IsARange bit at 16 is missing.
    #[test]
    fn vendor_range_missing_is_a_range_bit_returns_err() {
        assert_eq!(
            parse_vendor_range_from_bytes(&[0x00, 0x01], 4, &RangeSectionType::Vendor).err(),
            Some(TcsError::InsufficientLength)
        );
    }

    /// NumEntries = 1, IsARange = 1, but StartOrOnlyVendorId / EndVendorId cut short.
    #[test]
    fn vendor_range_truncated_range_entry_returns_err() {
        assert_eq!(
            parse_vendor_range_from_bytes(&[0x00, 0x1f, 0xff], 0, &RangeSectionType::Vendor).err(),
            Some(TcsError::InsufficientLength)
        );
    }

    /// NumEntries = 1, IsARange = 1, Start = 65534, End = 65535.
    #[test]
    fn vendor_range_ending_at_u16_max_includes_65535() {
        let section = parse_vendor_range_from_bytes(
            &[0x00, 0x1f, 0xff, 0xf7, 0xff, 0xf8],
            0,
            &RangeSectionType::Vendor,
        )
        .expect("range ending at 65535 must decode");

        assert_eq!(section.last_bit, 45);
        assert_eq!(vendor_list(section), vec![65534, 65535]);
    }

    #[test]
    fn bitfield_past_end_returns_err() {
        assert_eq!(
            parse_u16_bitfield_from_bytes(&[0xff], 0, 9),
            Err(TcsError::InsufficientLength)
        );
        assert_eq!(
            parse_u8_bitfield_from_bytes(&[0xff], 4, 5),
            Err(TcsError::InsufficientLength)
        );
    }
}
