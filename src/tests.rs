use crate::*;

#[test]
fn write_range_be() {
    let y = &mut [0b11111111u8, 0b11111111, 0b11111111, 0b11111111];
    let val = &[0b11111111u8, 0b11111111, 0b11110000, 0b00001111];

    y.range_write_be(4..12, 0);

    assert_eq!(&y, &val);
}

#[test]
fn write_range_le() {
    let y = &mut [0b11111111u8, 0b11111111, 0b11111111, 0b11111111];
    let val = &[0b00001111u8, 0b11110000, 0b11111111, 0b11111111];

    y.range_write_le(4..12, 0);

    assert_eq!(&y, &val);
}

#[test]
fn endian_check_le() {
    let y_le: u32 = 0b00001111_11110000_01010000_00001010;
    let y_arr = &[0b00001010u8, 0b01010000, 0b11110000, 0b00001111];
    assert_eq!(&y_le.to_le_bytes(), y_arr);

    let v: u32 = y_arr.range_read_le(..);
    assert_eq!(v, y_le);
}

#[test]
fn endian_check_be() {
    let y_be: u32 = 0b00001010_01010000_11110000_00001111;
    let y_arr_be = &[0b00001010u8, 0b01010000, 0b11110000, 0b00001111];
    assert_eq!(&y_be.to_be_bytes(), y_arr_be);

    let v: u32 = y_arr_be.range_read_be(..);
    assert_eq!(v, y_be);
}

#[test]
fn read_range_be() {
    // let y_be: u32 = 0b00001010_01010000_11110000_00001111;
    let y_arr = &[0b00001010u8, 0b01010000, 0b11110000, 0b00001111];

    let r: u32 = y_arr.range_read_be(0..8);
    assert_eq!(r, 0b00001111);

    let r: u32 = y_arr.range_read_be(0..16);
    assert_eq!(r, 0b11110000_00001111);

    let r: u32 = y_arr.range_read_be(0..24);
    assert_eq!(r, 0b01010000_11110000_00001111);
}


#[test]
fn read_range_le() {
    // let y: u32 = 0b00001111_11110001_11010011_00001110;
    let y = &[0b00001110u8, 0b11010011, 0b11110001, 0b10001111];

    let z3: u32 = y.range_read_le(8..24);

    assert_eq!(z3, 0b1111000111010011);

    let z3: u32 = y.range_read_le(0..8);
    assert_eq!(z3, 0b00001110);

    let arr = &[0x08, 0x00, 0x00, 0x00];
    let r: u32 = arr.range_read_le(0..32);
    assert_eq!(r, 0x08);

    let arr = &[0x63, 0x90, 0x6e, 0x3e, 0x1e, 0x75, 0xa6, 0xd7];
    let r: u64 = arr.range_read_le(0..64);
    println!("r: {:x}", r);
    assert_eq!(r, 0xd7a6_751e_3e6e_9063);
}
