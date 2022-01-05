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

// https://stackoverflow.com/questions/59602202/how-can-i-retain-vector-elements-with-their-original-index
pub fn with_index<T, F>(mut f: F) -> impl FnMut(&T) -> bool
where
    F: FnMut(usize, &T) -> bool,
{
    let mut i = 0;
    move |item| (f(i, item), i += 1).0
}

pub fn swap_remove_multiple<T>(vector: &mut Vec<T>, mut idx_to_remove: Vec<usize>) {
    idx_to_remove.sort();
    idx_to_remove.reverse();
    for idx in idx_to_remove {
        vector.swap_remove(idx);
    }
}
