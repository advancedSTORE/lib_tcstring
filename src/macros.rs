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
