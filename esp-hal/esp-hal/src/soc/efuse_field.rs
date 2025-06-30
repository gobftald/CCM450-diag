// 1
use core::{cmp, mem, slice};

// 3
use bytemuck::AnyBitPattern;
use console::panic;

// 5
use crate::soc::efuse::{Efuse, EfuseBlock};

/// The bit field for get access to efuse data
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 10
pub struct EfuseField {
    /// The block
    pub(crate) block: EfuseBlock,
    /// Word number - this is just informational
    pub(crate) word: u32,
    /// Starting bit in the efuse block
    pub(crate) bit_start: u32,
    /// Number of bits
    pub(crate) bit_count: u32,
}

// 21
impl EfuseField {
    pub(crate) const fn new(block: u32, word: u32, bit_start: u32, bit_count: u32) -> Self {
        Self {
            block: EfuseBlock::from_repr(block).unwrap(),
            word,
            bit_start,
            bit_count,
        }
    }
}

// 32
impl Efuse {
    /// Reads chip's MAC address from the eFuse storage.
    // 34
    pub fn read_base_mac_address() -> [u8; 6] {
        let mut mac_addr = [0u8; 6];

        let mac0 = Self::read_field_le::<[u8; 4]>(crate::soc::efuse::MAC0);
        let mac1 = Self::read_field_le::<[u8; 2]>(crate::soc::efuse::MAC1);

        // MAC address is stored in big endian, so load the bytes in reverse:
        mac_addr[0] = mac1[1];
        mac_addr[1] = mac1[0];
        mac_addr[2] = mac0[3];
        mac_addr[3] = mac0[2];
        mac_addr[4] = mac0[1];
        mac_addr[5] = mac0[0];

        mac_addr
    }

    /// Read field value in a little-endian order
    #[inline(always)]
    // 53
    pub fn read_field_le<T: AnyBitPattern>(field: EfuseField) -> T {
        let EfuseField {
            block,
            bit_start,
            bit_count,
            ..
        } = field;

        // Represent output value as a bytes slice:
        let mut output = mem::MaybeUninit::<T>::uninit();
        let mut bytes = unsafe {
            slice::from_raw_parts_mut(output.as_mut_ptr() as *mut u8, mem::size_of::<T>())
        };

        let bit_off = bit_start as usize;
        let bit_end = cmp::min(bit_count as usize, bytes.len() * 8) + bit_off;

        let mut last_word_off = bit_off / 32;
        let mut last_word = unsafe { block.address().add(last_word_off).read_volatile() };

        let word_bit_off = bit_off % 32;
        let word_bit_ext = 32 - word_bit_off;

        let mut word_off = last_word_off;
        for bit_off in (bit_off..bit_end).step_by(32) {
            if word_off != last_word_off {
                // Read a new word:
                last_word_off = word_off;
                last_word = unsafe { block.address().add(last_word_off).read_volatile() };
            }

            let mut word = last_word >> word_bit_off;
            word_off += 1;

            let word_bit_len = cmp::min(bit_end - bit_off, 32);
            if word_bit_len > word_bit_ext {
                // Read the next word:
                last_word_off = word_off;
                last_word = unsafe { block.address().add(last_word_off).read_volatile() };
                // Append bits from a beginning of the next word:
                word |= last_word.wrapping_shl((32 - word_bit_off) as u32);
            };

            if word_bit_len < 32 {
                // Mask only needed bits of a word:
                word &= u32::MAX >> (32 - word_bit_len);
            }

            // Represent word as a byte slice:
            let byte_len = word_bit_len.div_ceil(8);
            let word_bytes =
                unsafe { slice::from_raw_parts(&word as *const u32 as *const u8, byte_len) };

            // Copy word bytes to output value bytes:
            bytes[..byte_len].copy_from_slice(word_bytes);

            // Move read window forward:
            bytes = &mut bytes[byte_len..];
        }

        // Fill untouched bytes with zeros:
        bytes.fill(0);

        unsafe { output.assume_init() }
    }
}
