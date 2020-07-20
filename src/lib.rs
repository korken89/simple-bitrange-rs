//! # A simple bit range crate
//!
//! This crate aims to facilitate the extraction of bits in a small, simple crate. While it does
//! not have as many bells and whistle as many other crates, simplicity is the key here.
//!
//! # Usage examples
//!
//! Extract bits from slice of bytes:
//!
//! ```rust
//! # use bitrange::*;
//! let y: u32 = 0b00001111_11110000_01010000_00001010;
//! let p: &[u8] = &y.to_le_bytes();
//! let ret: u32 = p.range_read(..);
//!
//! assert_eq!(ret, y);
//! ```

// #![no_std]
#![deny(missing_docs)]

use core::mem::size_of;
use core::ops::{Bound, RangeBounds};

/// A simple bit extraction definition.
pub trait BitRangeRead<U> {
    /// Reads a range of bits from the type.
    fn range_read<R: RangeBounds<usize>>(self, range: R) -> U;
}

/// A simple bit write definition.
pub trait BitRangeWrite<U> {
    /// Writes a range of bits into a specific range.
    fn range_write<R: RangeBounds<usize>>(self, range: R, value: U);
}

macro_rules! impl_bit_range {
    ($($numeric:ty,)*) => {$(
        impl BitRangeRead<$numeric> for $numeric {
            fn range_read<R: RangeBounds<usize>>(self, range: R) -> $numeric {
                let masked = match range.end_bound() {
                    Bound::Included(end) => self & ((1 << (*end + 1)) - 1),
                    Bound::Excluded(end) => self & ((1 << *end) - 1),
                    Bound::Unbounded => self,
                };

                match range.start_bound() {
                    Bound::Included(start) => masked >> *start,
                    Bound::Excluded(start) => masked >> (*start + 1),
                    Bound::Unbounded => masked,
                }
            }
        })*
    }
}

// Basic type implementations
impl_bit_range!(u8, u16, u32, u64, u128,);

macro_rules! impl_bit_range_slice {
    ($($numeric:ty,)*) => {$(
        impl BitRangeRead<$numeric> for &'_ [u8] {
            #[cfg_attr(feature = "enable-inline", inline)]
            #[cfg_attr(feature = "never-inline", inline(never))]
            fn range_read<R: RangeBounds<usize>>(self, range: R) -> $numeric {
                let res: u64 = bit_range_read_impl(&self, range);
                res as $numeric
            }
        })*
    }
}

// Slice implementations
impl_bit_range_slice!(u8, u16, u32, u64,);

macro_rules! impl_bit_range_write_slice {
    ($($numeric:ty,)*) => {$(
        impl BitRangeWrite<$numeric> for &'_ mut [u8] {
            #[cfg_attr(feature = "enable-inline", inline)]
            #[cfg_attr(feature = "never-inline", inline(never))]
            fn range_write<R: RangeBounds<usize>>(self, range: R, value: $numeric) {
                bit_range_write_impl(self, value as u64, range)
            }
        })*
    }
}

// Slice implementations
impl_bit_range_write_slice!(u8, u16, i32, u32, u64,);

// try 2
#[inline]
fn bit_range_read_impl<R: RangeBounds<usize>>(input: &[u8], range: R) -> u64 {
    // Let's figure out where the bytes are located within the array.
    let start_bit = match range.start_bound() {
        Bound::Included(start) => *start,
        Bound::Excluded(start) => *start + 1,
        Bound::Unbounded => 0,
    };
    let end_bit = match range.end_bound() {
        Bound::Included(end) => *end,
        Bound::Excluded(end) => *end - 1,
        Bound::Unbounded => size_of::<u64>() * 8 - 1,
    };
    let total_bits = end_bit - start_bit + 1;
    let start_byte = start_bit / 8;
    let end_byte = (end_bit / 8).min(input.len());

    let start_bit = start_bit - start_byte * 8;

    // Check safety
    assert!(total_bits <= 64);

    // The rust compiler is smart enough to see through this and not so u128 operations.
    let mask = (1 << total_bits) - 1;
    let mut output = read_le_u128(unsafe { get_unchecked(start_byte, end_byte + 1, input) });
    output >>= start_bit;
    output &= mask;

    output as u64
}
fn bit_range_write_impl<R: RangeBounds<usize>>(output: &mut [u8], val: u64, range: R) {
    // Let's figure out where the bytes are located within the array.
    let start_bit = match range.start_bound() {
        Bound::Included(start) => *start,
        Bound::Excluded(start) => *start + 1,
        Bound::Unbounded => 0,
    };
    let end_bit = match range.end_bound() {
        Bound::Included(end) => *end,
        Bound::Excluded(end) => *end - 1,
        Bound::Unbounded => size_of::<u64>() * 8 - 1,
    };
    let total_bits = end_bit - start_bit + 1;
    let start_byte = start_bit / 8;
    let end_byte = (end_bit / 8).min(output.len());

    let start_bit = start_bit - start_byte * 8;

    assert!(total_bits <= 64);

    // Extract area as u128
    let mut work_value = read_le_u128(unsafe { get_unchecked(start_byte, end_byte + 1, output) });

    let mask = ((1 << total_bits) - 1) << start_bit;
    let val = ((val as u128) << start_bit) & mask;

    // Modify area
    work_value &= !mask;
    work_value |= val;

    // Write area back
    unsafe { write_value(start_byte, end_byte + 1, output, work_value) };
}

#[inline(always)]
unsafe fn get_unchecked<T>(start: usize, end: usize, slice: &[T]) -> &[T] {
    core::slice::from_raw_parts(slice.as_ptr().add(start), end - start)
}

#[inline]
fn read_le_u128(input: &[u8]) -> u128 {
    let mut ret = 0;

    for b in input.iter().rev() {
        ret <<= 8;
        ret |= *b as u128;
    }

    ret
}

#[inline(always)]
unsafe fn write_value(start: usize, end: usize, output: &mut [u8], value: u128) {
    let val_as_bytes = &value.to_le_bytes();
    core::ptr::copy_nonoverlapping(
        val_as_bytes.as_ptr(),
        output.as_mut_ptr().add(start),
        end - start,
    );
}

#[cfg(test)]
#[macro_use]
extern crate std;

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn write_range() {
        let y = &mut [0b11111111u8, 0b11111111, 0b11111111, 0b11111111];
        let val = &[0b00001111u8, 0b11110000, 0b11111111, 0b11111111];

        // println!("");

        y.range_write(4..12, 0);

        // for (i, b) in y.iter().enumerate() {
        //     println!("{}: {:#010b}", i, *b);
        // }

        assert_eq!(&y, &val);
    }

    #[test]
    fn endian_check() {
        let y: u32 = 0b00001111_11110000_01010000_00001010;
        let y_arr = &[0b00001010u8, 0b01010000, 0b11110000, 0b00001111];
        assert_eq!(&y.to_le_bytes(), y_arr);

        let v: u32 = y_arr.range_read(..);
        assert_eq!(v, y);
    }

    #[test]
    fn read_range() {
        // let x: u32 = 0b000111100;

        // println!("");
        // println!("x: {:b}", x);
        // println!("1: {:b}", x.range_read(..));
        // println!("2: {:b}", x.range_read(..3));
        // println!("3: {:b}", x.range_read(..=3));
        // println!("4: {:b}", x.range_read(1..));
        // // println!("{:b}", x.range_read(18446744073709551615..));
        // println!("5: {:b}", x.range_read(2..5));
        // println!("6: {:b}", x.range_read(2..=5));
        // // println!("{:b}", x.range_read(2..=5234234234234234));

        // let y: u32 = 0b00001111_11110001_11010011_00001110;
        let y = &[0b00001110u8, 0b11010011, 0b11110001, 0b10001111];

        let z3: u64 = y.range_read(8..24);

        assert_eq!(z3, 0b1111000111010011);
    }
}
