use crate::eth::RX_BUF_LENGTH;
use arr_macro::arr;

//#[derive(Debug)]
pub struct ControlBlock {
    /// Index of the hw descriptor, `RxDescriptor` or `TxDescriptor`
    pub(crate) desc_index: usize,
    /// Frame buffer
    pub(crate) buffer: [u8; RX_BUF_LENGTH],
}

impl ControlBlock {
    pub const fn zero() -> Self {
        ControlBlock {
            desc_index: 0,
            buffer: arr![0; 2048],
        }
    }
}
