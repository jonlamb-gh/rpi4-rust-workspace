use crate::eth::RX_BUF_LENGTH;
use arr_macro::arr;
use core::fmt;
use static_assertions::const_assert_eq;

const_assert_eq!(RX_BUF_LENGTH, 2048);

/// Hardware descriptor
pub struct Descriptor {
    /// Frame buffer
    pub(crate) buffer: [u8; RX_BUF_LENGTH],
}

impl Descriptor {
    pub const fn zero() -> Self {
        Descriptor {
            buffer: arr![0; 2048],
        }
    }

    #[inline]
    pub(crate) fn as_slice(&self) -> &[u8] {
        &self.buffer[..]
    }

    #[inline]
    pub(crate) fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.buffer[..]
    }
}

impl fmt::Display for Descriptor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Descriptor {{ buffer len {} at 0x{:X} }}",
            self.buffer.len(),
            self.buffer.as_ptr() as usize
        )
    }
}
