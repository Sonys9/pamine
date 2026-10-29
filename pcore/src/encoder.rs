pub fn varint(mut number: u32) -> Vec<u8> {
    let mut array = Vec::with_capacity(5);
    loop {
        let byte = number & 0b1111111;
        number >>= 7;
        match number != 0 {
            true => array.push(byte as u8 | 0b10000000),
            false => {
                array.push(byte as u8);
                break;
            }
        }
    }
    array
}