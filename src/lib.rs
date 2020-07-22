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
//! # use simple_bitrange::*;
//! let y: u32 = 0b00001111_11110000_01010000_00001010;
//! let p: &[u8] = &y.to_le_bytes();
//! let ret: u32 = p.range_read_le(..);
//!
//! assert_eq!(ret, y);
//! ```

#![no_std]
#![deny(missing_docs)]

use core::iter::{DoubleEndedIterator, ExactSizeIterator};
use core::mem::size_of;
use core::ops::{Bound, RangeBounds};

/// A simple bit extraction definition.
pub trait BitRangeRead<U> {
    // /// Reads a range of bits from the type in native endian.
    // fn range_read_ne<R: RangeBounds<usize>>(self, range: R) -> U;
    /// Reads a range of bits from the type in little endian.
    fn range_read_le<R: RangeBounds<usize>>(self, range: R) -> U;
    /// Reads a range of bits from the type in big endian.
    fn range_read_be<R: RangeBounds<usize>>(self, range: R) -> U;
}

/// A simple bit write definition.
pub trait BitRangeWrite<U> {
    // /// Writes a range of bits into a specific range using native endian.
    // fn range_write_ne<R: RangeBounds<usize>>(self, range: R, value: U);
    /// Writes a range of bits into a specific range using little endian.
    fn range_write_le<R: RangeBounds<usize>>(self, range: R, value: U);
    /// Writes a range of bits into a specific range using big endian.
    fn range_write_be<R: RangeBounds<usize>>(self, range: R, value: U);
}

// macro_rules! impl_bit_range {
//     ($($numeric:ty,)*) => {$(
//         impl BitRangeRead<$numeric> for $numeric {
//             fn range_read<R: RangeBounds<usize>>(self, range: R) -> $numeric {
//                 let masked = match range.end_bound() {
//                     Bound::Included(end) => self & ((1 << (*end + 1)) - 1),
//                     Bound::Excluded(end) => self & ((1 << *end) - 1),
//                     Bound::Unbounded => self,
//                 };
//
//                 match range.start_bound() {
//                     Bound::Included(start) => masked >> *start,
//                     Bound::Excluded(start) => masked >> (*start + 1),
//                     Bound::Unbounded => masked,
//                 }
//             }
//         })*
//     }
// }
//
// // Basic type implementations
// impl_bit_range!(u8, u16, u32, u64, u128,);

macro_rules! impl_bit_range_slice {
    ($($numeric:ty,)*) => {$(
        impl BitRangeRead<$numeric> for &'_ [u8] {
            #[cfg_attr(feature = "enable-inline", inline)]
            #[cfg_attr(feature = "never-inline", inline(never))]
            fn range_read_le<R: RangeBounds<usize>>(self, range: R) -> $numeric {
                let res: u64 = bit_range_read_le_iter_impl(self.iter(), range);
                res as $numeric
            }

            #[cfg_attr(feature = "enable-inline", inline)]
            #[cfg_attr(feature = "never-inline", inline(never))]
            fn range_read_be<R: RangeBounds<usize>>(self, range: R) -> $numeric {
                let res: u64 = bit_range_read_le_iter_impl(self.iter().rev(), range);
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
            fn range_write_le<R: RangeBounds<usize>>(self, range: R, value: $numeric) {
                write_le_compound(self, value as u64, range);
            }

            #[cfg_attr(feature = "enable-inline", inline)]
            #[cfg_attr(feature = "never-inline", inline(never))]
            fn range_write_be<R: RangeBounds<usize>>(self, range: R, value: $numeric) {
                write_be_compound(self, value as u64, range);
            }
        })*
    }
}

// Slice implementations
impl_bit_range_write_slice!(u8, u16, i32, u32, u64,);

/// Helper of common code
#[inline(always)]
fn setup_iter<'a, R>(input_len: usize, range: R) -> (usize, usize, usize, usize)
where
    R: RangeBounds<usize>,
{
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
    let end_byte = (end_bit / 8).min(input_len);

    let start_bit = start_bit - start_byte * 8;

    (start_bit, total_bits, start_byte, end_byte)
}


#[cfg_attr(feature = "enable-inline", inline)]
#[cfg_attr(feature = "never-inline", inline(never))]
fn bit_range_read_le_iter_impl<'a, I, R>(input: I, range: R) -> u64
where
    I: Iterator<Item = &'a u8> + DoubleEndedIterator + ExactSizeIterator,
    R: RangeBounds<usize>,
{
    let (start_bit, total_bits, start_byte, end_byte) = setup_iter(input.len(), range);

    let iter = input.skip(start_byte).take(end_byte + 1);

    // The rust compiler is smart enough to see through this and not so u128 operations.
    let mask = (1 << total_bits) - 1;
    let mut output = read_u128_le(iter);
    output >>= start_bit;
    output &= mask;

    output as u64
}

#[cfg_attr(feature = "enable-inline", inline)]
#[cfg_attr(feature = "never-inline", inline(never))]
fn write_le_compound<R>(output: &mut [u8], val: u64, range: R)
where
    R: RangeBounds<usize>,
{
    let (start_bit, total_bits, start_byte, end_byte) = setup_iter(output.len(), range);
    let iter = output.iter().skip(start_byte).take(end_byte + 1);

    // Extract area as u128
    let mut work_value = read_u128_le(iter);

    let mask = ((1 << total_bits) - 1) << start_bit;
    let val = ((val as u128) << start_bit) & mask;

    // Modify area
    work_value &= !mask;
    work_value |= val;

    let iter = output.iter_mut().skip(start_byte).take(end_byte + 1);

    // Write area back
    write_value_le(iter, work_value);
}

#[cfg_attr(feature = "enable-inline", inline)]
#[cfg_attr(feature = "never-inline", inline(never))]
fn write_be_compound<R>(output: &mut [u8], val: u64, range: R)
where
    R: RangeBounds<usize>,
{
    let (start_bit, total_bits, start_byte, end_byte) = setup_iter(output.len(), range);
    let iter = output.iter().rev().skip(start_byte).take(end_byte + 1);

    // Extract area as u128
    let mut work_value = read_u128_le(iter);

    let mask = ((1 << total_bits) - 1) << start_bit;
    let val = ((val as u128) << start_bit) & mask;

    // Modify area
    work_value &= !mask;
    work_value |= val;

    let iter = output.iter_mut().rev().skip(start_byte).take(end_byte + 1);

    // Write area back
    write_value_le(iter, work_value);
}

#[inline(always)]
fn read_u128_le<'a, I>(input: I) -> u128
where
    I: Iterator<Item = &'a u8> + DoubleEndedIterator,
{
    input.rev().fold(0, |acc, x| (acc << 8) | *x as u128)
}

#[inline(always)]
fn write_value_le<'a, O>(output: O, value: u128)
where
    O: Iterator<Item = &'a mut u8> + DoubleEndedIterator,
{
    let val_as_bytes = &value.to_be_bytes();
    val_as_bytes
        .iter()
        .rev()
        .zip(output)
        .for_each(|(i, o)| *o = *i);
}

#[cfg(test)]
#[macro_use]
extern crate std;

#[cfg(test)]
mod tests;
