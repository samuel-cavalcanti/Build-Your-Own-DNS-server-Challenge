pub fn double_u8_to_u16(bytes: &[u8], i: usize) -> u16 {
    (bytes[i] as u16) << 8 | (bytes[i + 1] as u16)
}
pub fn bytes_to_i32(bytes: &[u8], i: usize) -> i32 {
    (bytes[i] as i32) << (8 * 3)
        | (bytes[i + 1] as i32) << (8 * 2)
        | (bytes[i + 2] as i32) << 8
        | (bytes[i + 3] as i32)
}
