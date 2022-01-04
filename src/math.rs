pub fn floor(x: u32, y: u32) -> u32 {
    x / y
}

pub fn ceil(x: u32, y: u32) -> u32 {
    (x + y - 1) / y
}

pub fn mask(bit_offset: u32, bit_len: u32) -> u32 {
    let last_bit = bit_offset + bit_len;
    assert!((last_bit <= 32) && (bit_len < 32), "wrong mask for bit_offset {}, bit_len {}", bit_offset, bit_len);
    ((1 << bit_len) - 1) << bit_offset
}
