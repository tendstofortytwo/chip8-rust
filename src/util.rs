// given a hex number, return number formed by taking
// d digits from it after removing the first o digits
// looking from least significant digit first
// eg. get_hex_digits(0x1e90ff, 3, 2) will return 0xe90:
// it removes first 2 digits (ff) and then returns the
// number formed by taking the next 3 (0xe90)
pub fn get_hex_digits(n: &u16, d: u32, o: u32) -> usize {
    let base: u16 = 0x10;
    ((n / base.pow(o)) % base.pow(d)) as usize
}

// check if nth bit of a byte is set,
// zero-indexed, least significant first
pub fn is_bit_set(byte: &u8, n: u8) -> bool {
    if byte & (1 << n) == 0 { false } else { true }
}

// return nth bit of a byte, zero-indexed, 
// least significant first
pub fn get_bit(byte: &u8, n: u8) -> u8 {
    if is_bit_set(byte, n) { 1 } else { 0 }
}
