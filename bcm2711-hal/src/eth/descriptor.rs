use crate::eth::RX_BUF_LENGTH;
use arr_macro::arr;
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
}
