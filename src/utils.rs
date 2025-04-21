pub fn double_u8_to_u16(bytes: &[u8], i: usize) -> u16 {
    (bytes[i] as u16) << 8 | (bytes[i + 1] as u16)
}
