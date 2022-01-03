pub fn floor(x: u32, y: u32) -> u32 {
    x / y
}

pub fn ceil(x: u32, y: u32) -> u32 {
    (x + y - 1) / y
}
