/// Ring of Rx buffers
#[derive(Debug)]
pub struct RxRing {
    /// Ring index
    pub(crate) index: usize,
    /// Ring buffer control block
    pub(crate) cbs_index: usize,
    /// Ring size
    pub(crate) size: usize,
    /// Last consumer index
    pub(crate) c_index: usize,
    /// Ring read pointer
    pub(crate) read_ptr: usize,
    /// Ring initial control block pointer
    pub(crate) cb_ptr: usize,
    /// Ring end control block pointer
    pub(crate) end_ptr: usize,
    /// Old discards
    pub(crate) old_discards: usize,
}

impl RxRing {
    pub const fn zero() -> Self {
        RxRing {
            index: 0,
            cbs_index: 0,
            size: 0,
            c_index: 0,
            read_ptr: 0,
            cb_ptr: 0,
            end_ptr: 0,
            old_discards: 0,
        }
    }
}
