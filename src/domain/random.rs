



pub(crate) fn next_u64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);

    let mut value = *state;
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}


pub(crate) fn unit_f32(state: &mut u64) -> f32 {
    let mantissa = (next_u64(state) >> 40) as u32;
    mantissa as f32 * (1.0 / 16_777_216.0)
}

