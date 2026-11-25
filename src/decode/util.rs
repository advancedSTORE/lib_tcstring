use crate::decode::{
    error::TcsError,
    model::{RangeSection, RangeSectionType},
};

/// Maximum number of vendor IDs one range-encoded section may expand to.
///
/// Range entries are cheap to encode (33 bits for up to 65,535 IDs), so an
/// untrusted TC string can otherwise request hundreds of millions of IDs.
/// This caps the work and memory a single string can cause (DEV-11330).
///
/// * Vendor consent, vendor LI, Disclosed Vendors and Allowed Vendors: the
///   limit applies to each section's expanded vendor list.
/// * Publisher restrictions: the section has no `MaxVendorId`, so the limit
///   applies to the sum of all restrictions' vendor lists. The total is
///   checked after each restriction is expanded, so peak memory before
///   rejection is at most about twice the limit.
///
/// 65,536 is one full set of 16-bit vendor IDs, so it is never smaller than
/// any `MaxVendorId` and is far above what real CMPs produce
/// (about 11 purposes × the size of the Global Vendor List). With `u16` IDs
/// this is about 128 KB per section.
pub(crate) const VENDOR_ID_NUMBER_LIMIT: usize = 65536;

pub(crate) fn parse_from_bytes(val: &[u8], absolute_start_bit: usize, bit_length: usize) -> u64 {
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
        return return_value;
    }

    current_byte += 1;

    while current_byte < last_byte_index {
        return_value = (return_value << 8) | (val[current_byte] as u64);
        current_byte += 1;
    }

    let bit_shift = 7 - last_byte_end_bit;

    (return_value << (last_byte_end_bit + 1))
        | ((val[current_byte] as u64 & (0xff << bit_shift)) >> bit_shift)
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
        let alphabet_offset = parse_from_bytes(val, bit_start + offset, bit_width) as u8;

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
    max_vendor_id: Option<u16>,
) -> Result<RangeSection, TcsError> {
    let mut bit_index = bit_start + 12;

    byte_list_bit_boundary_check!(val, bit_index);

    let num_entries = parse_from_bytes(val, bit_start, 12) as u16;
    let cap = precount_entry_list(val, bit_index, num_entries, max_vendor_id)?;
    let mut entry_list: Vec<u16> = Vec::with_capacity(cap);
    let mut count = 0u16;

    while count < num_entries {
        if parse_from_bytes(val, bit_index, 1) as u8 == 1 {
            byte_list_bit_boundary_check!(val, bit_index + 33);

            let start_vendor_id = parse_from_bytes(val, bit_index + 1, 16) as u16;
            let end_vendor_id = parse_from_bytes(val, bit_index + 17, 16) as u16;

            for vendor_id in start_vendor_id..end_vendor_id + 1 {
                entry_list.push(vendor_id);
            }

            bit_index += 33;
        } else {
            byte_list_bit_boundary_check!(val, bit_index + 17);

            entry_list.push(parse_from_bytes(val, bit_index + 1, 16) as u16);
            bit_index += 17;
        }

        count += 1;
    }

    Ok(RangeSection {
        last_bit: bit_index,
        value: value_type(entry_list),
    })
}

fn precount_entry_list(
    val: &[u8],
    bit_index: usize,
    num_entries: u16,
    max_vendor_id: Option<u16>,
) -> Result<usize, TcsError> {
    let mut bit_index = bit_index;
    let mut entry_count = 0usize;
    let mut count = 0u16;

    while count < num_entries {
        if parse_from_bytes(val, bit_index, 1) as u8 == 1 {
            byte_list_bit_boundary_check!(val, bit_index + 33);

            let start_vendor_id = parse_from_bytes(val, bit_index + 1, 16) as u16;
            let end_vendor_id = parse_from_bytes(val, bit_index + 17, 16) as u16;

            validate_range_vendor_id(start_vendor_id, end_vendor_id, max_vendor_id)?;

            entry_count += usize::from(end_vendor_id - start_vendor_id) + 1;
            bit_index += 33;
        } else {
            byte_list_bit_boundary_check!(val, bit_index + 17);

            entry_count += 1;
            bit_index += 17;
        }

        if entry_count > VENDOR_ID_NUMBER_LIMIT {
            return Err(TcsError::InvalidSectionDefinition);
        }

        count += 1;
    }

    Ok(entry_count)
}

fn validate_range_vendor_id(start: u16, end: u16, max: Option<u16>) -> Result<(), TcsError> {
    if let Some(max) = max {
        if end > max {
            return Err(TcsError::InvalidSectionDefinition);
        }
    }

    if start > end {
        return Err(TcsError::InvalidSectionDefinition);
    }

    Ok(())
}

parse_bitfield_from_bytes!(parse_u8_bitfield_from_bytes, u8);
parse_bitfield_from_bytes!(parse_u16_bitfield_from_bytes, u16);

/// Helpers shared by the decoder tests in this crate.
#[cfg(test)]
pub(crate) mod test_support {
    use crate::decode::error::TcsError;

    /// Builds a byte buffer bit by bit, most significant bit first,
    /// which is the order `parse_from_bytes` reads in.
    #[derive(Default)]
    pub(crate) struct BitWriter {
        bytes: Vec<u8>,
        bit_len: usize,
    }

    impl BitWriter {
        pub(crate) fn new() -> Self {
            Self::default()
        }

        /// Appends the lowest `width` bits of `value`.
        pub(crate) fn push(&mut self, value: u64, width: usize) -> &mut Self {
            for i in (0..width).rev() {
                let bit_in_byte = self.bit_len % 8;

                if bit_in_byte == 0 {
                    self.bytes.push(0);
                }

                if (value >> i) & 1 == 1 {
                    let last = self.bytes.len() - 1;
                    self.bytes[last] |= 0x80 >> bit_in_byte;
                }

                self.bit_len += 1;
            }

            self
        }

        /// Appends a single-ID entry: `IsARange = 0`, then a 16-bit ID (17 bits).
        pub(crate) fn single(&mut self, id: u16) -> &mut Self {
            self.push(0, 1).push(u64::from(id), 16)
        }

        /// Appends a range entry: `IsARange = 1`, then 16-bit start and end (33 bits).
        pub(crate) fn range(&mut self, start: u16, end: u16) -> &mut Self {
            self.push(1, 1)
                .push(u64::from(start), 16)
                .push(u64::from(end), 16)
        }

        pub(crate) fn bit_len(&self) -> usize {
            self.bit_len
        }

        pub(crate) fn to_bytes(&self) -> Vec<u8> {
            self.bytes.clone()
        }
    }

    /// Asserts `Err(InvalidSectionDefinition)` without printing the `Ok` value,
    /// which can contain millions of vendor IDs.
    pub(crate) fn assert_invalid_section<T>(result: Result<T, TcsError>) {
        match result {
            Err(err) => assert_eq!(err, TcsError::InvalidSectionDefinition),
            Ok(_) => panic!("expected Err(InvalidSectionDefinition), got Ok"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::{assert_invalid_section, BitWriter};
    use super::*;

    // validate_range_vendor_id

    #[test]
    fn validate_rejects_start_greater_than_end() {
        assert_invalid_section(validate_range_vendor_id(8, 5, None));
        assert_invalid_section(validate_range_vendor_id(8, 5, Some(100)));
    }

    #[test]
    fn validate_rejects_end_greater_than_max() {
        assert_invalid_section(validate_range_vendor_id(1, 11, Some(10)));
    }

    #[test]
    fn validate_accepts_end_equal_to_max() {
        assert_eq!(validate_range_vendor_id(1, 10, Some(10)), Ok(()));
    }

    #[test]
    fn validate_accepts_start_equal_to_end() {
        assert_eq!(validate_range_vendor_id(3, 3, Some(10)), Ok(()));
    }

    #[test]
    fn validate_without_max_only_checks_order() {
        assert_eq!(validate_range_vendor_id(1, 65535, None), Ok(()));
    }

    // precount_entry_list

    #[test]
    fn precount_counts_ranges_inclusively_and_singles_as_one() {
        // 3..=7 is 5 IDs, plus 1 single ID = 6
        let bytes = BitWriter::new().range(3, 7).single(20).to_bytes();

        assert_eq!(precount_entry_list(&bytes, 0, 2, Some(20)), Ok(6));
    }

    #[test]
    fn precount_rejects_start_greater_than_end() {
        let bytes = BitWriter::new().range(8, 5).to_bytes();

        assert_invalid_section(precount_entry_list(&bytes, 0, 1, None));
    }

    #[test]
    fn precount_rejects_end_greater_than_max() {
        let bytes = BitWriter::new().range(1, 11).to_bytes();

        assert_invalid_section(precount_entry_list(&bytes, 0, 1, Some(10)));
    }

    #[test]
    fn precount_full_u16_range_does_not_overflow() {
        // 0..=65535 is 65,536 IDs, one more than u16::MAX
        let bytes = BitWriter::new().range(0, 65535).to_bytes();

        assert_eq!(precount_entry_list(&bytes, 0, 1, None), Ok(65_536));
    }

    #[test]
    fn precount_accepts_exactly_the_limit() {
        // 2 × 32,768 IDs = 65,536 = VENDOR_ID_NUMBER_LIMIT
        let bytes = BitWriter::new().range(0, 32767).range(0, 32767).to_bytes();

        assert_eq!(
            precount_entry_list(&bytes, 0, 2, None),
            Ok(VENDOR_ID_NUMBER_LIMIT)
        );
    }

    #[test]
    fn precount_rejects_one_range_id_past_the_limit() {
        // 65,536 IDs, then one more range ID
        let bytes = BitWriter::new().range(0, 65535).range(1, 1).to_bytes();

        assert_invalid_section(precount_entry_list(&bytes, 0, 2, None));
    }

    #[test]
    fn precount_rejects_one_single_id_past_the_limit() {
        // 65,536 IDs, then one single-ID entry
        let bytes = BitWriter::new().range(0, 65535).single(1).to_bytes();

        assert_invalid_section(precount_entry_list(&bytes, 0, 2, None));
    }

    #[test]
    fn precount_limit_is_not_bypassed_by_u16_wrap_around() {
        // 2 × 65,534 = 131,068 IDs. Truncated to u16 this would be 65,532,
        // which is under the limit, so the comparison must stay in usize
        let bytes = BitWriter::new().range(1, 65534).range(1, 65534).to_bytes();

        assert_invalid_section(precount_entry_list(&bytes, 0, 2, Some(65534)));
    }

    #[test]
    fn precount_stops_at_the_first_entry_over_the_limit() {
        // Entry 2 goes over the limit; entry 3 is missing from the input.
        // Returning InvalidSectionDefinition (not InsufficientLength) shows
        // the precount stopped before reading further
        let bytes = BitWriter::new().range(0, 65535).range(0, 65535).to_bytes();

        assert_invalid_section(precount_entry_list(&bytes, 0, 3, None));
    }

    #[test]
    fn vendor_range_over_the_limit_is_rejected() {
        let bytes = BitWriter::new()
            .push(2, 12)
            .range(1, 65534)
            .range(1, 65534)
            .to_bytes();

        assert_invalid_section(parse_vendor_range_from_bytes(
            &bytes,
            0,
            &RangeSectionType::Vendor,
            Some(65534),
        ));
    }

    // parse_vendor_range_from_bytes

    #[test]
    fn vendor_range_decodes_valid_entries_unchanged() {
        let mut writer = BitWriter::new();
        writer.push(2, 12).single(2).range(5, 8);
        let bytes = writer.to_bytes();

        let section =
            parse_vendor_range_from_bytes(&bytes, 0, &RangeSectionType::Vendor, Some(10)).unwrap();

        // 12 (NumEntries) + 17 (single) + 33 (range)
        assert_eq!(section.last_bit, writer.bit_len());
        assert_eq!(section.last_bit, 62);
        match section.value {
            RangeSectionType::Vendor(ids) => assert_eq!(ids, vec![2, 5, 6, 7, 8]),
            other => panic!("unexpected section type: {:?}", other),
        }
    }

    #[test]
    fn vendor_range_rejects_start_greater_than_end() {
        let bytes = BitWriter::new().push(1, 12).range(8, 5).to_bytes();

        assert_invalid_section(parse_vendor_range_from_bytes(
            &bytes,
            0,
            &RangeSectionType::Vendor,
            Some(10),
        ));
    }

    #[test]
    fn vendor_range_rejects_end_greater_than_max() {
        let bytes = BitWriter::new().push(1, 12).range(1, 11).to_bytes();

        assert_invalid_section(parse_vendor_range_from_bytes(
            &bytes,
            0,
            &RangeSectionType::Vendor,
            Some(10),
        ));
    }

    #[test]
    fn vendor_range_truncated_input_is_insufficient_length() {
        // NumEntries says 2, but only one entry follows
        let bytes = BitWriter::new().push(2, 12).range(1, 5).to_bytes();

        match parse_vendor_range_from_bytes(&bytes, 0, &RangeSectionType::Vendor, Some(10)) {
            Err(err) => assert_eq!(err, TcsError::InsufficientLength),
            Ok(_) => panic!("expected Err(InsufficientLength), got Ok"),
        }
    }
}
