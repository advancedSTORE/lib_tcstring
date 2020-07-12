#[macro_export]
macro_rules! byte_list_bit_boundary_check {
    ($byte_list: expr, $bit_index: expr) => {{
        let length = $byte_list.len();
        let bit_index: usize = $bit_index;

        if length * 8 <= bit_index {
            return Err(crate::decode::error::INSUFFICIENT_LENGTH);
        }
    }};
}

#[macro_export]
macro_rules! range_section_value {
    ($sections: expr, $variant: path) => {{
        let sections = &mut $sections;

        if sections.is_empty() {
            return Err(crate::decode::error::INVALID_SECTION_DEFINTION);
        }

        if let RangeSection {
            last_bit: _,
            value: $variant(section),
        } = sections.remove(0)
        {
            section
        } else {
            return Err(crate::decode::error::INVALID_SECTION_DEFINTION);
        }
    }};
}

#[macro_export]
macro_rules! parse_bitfield_from_bytes {
    ($name: ident, $type: tt) => {
        pub(crate) fn $name(
            val: &[u8],
            bit_start: usize,
            bit_length: usize,
        ) -> Result<Vec<$type>, crate::decode::model::TCSDecodeError> {
            let bit_end = bit_start + bit_length;
            byte_list_bit_boundary_check!(val, bit_end - 1);
            let mut result: Vec<$type> = Vec::with_capacity(bit_length);
            for bit_index in bit_start..bit_end {
                if parse_from_bytes(val, bit_index, 1) == 1 {
                    result.push(((bit_index - bit_start) + 1) as $type);
                }
            }
            Ok(result)
        }
    };
}
