/// Ring of Tx buffers
#[derive(Debug)]
pub struct TxRing {
    /// Ring index
    pub(crate) index: usize,
    /// Queue index
    pub(crate) queue: usize,
    /// Ring buffer control block
    pub(crate) cbs_index: usize,
    /// Ring size
    pub(crate) size: usize,
    /// Ring clean pointer
    pub(crate) clean_ptr: usize,
    /// Last consumer index
    pub(crate) c_index: usize,
    /// Number of free buffer descriptors for each ring
    pub(crate) free_bds: usize,
    /// Ring write pointer sw copy
    pub(crate) write_ptr: usize,
    /// Producer index sw copy
    pub(crate) prod_index: usize,
    /// Ring initial control block pointer
    pub(crate) cb_ptr: usize,
    /// Ring end control block pointer
    pub(crate) end_ptr: usize,
}

impl TxRing {
    pub const fn zero() -> Self {
        TxRing {
            index: 0,
            queue: 0,
            cbs_index: 0,
            size: 0,
            clean_ptr: 0,
            c_index: 0,
            free_bds: 0,
            write_ptr: 0,
            prod_index: 0,
            cb_ptr: 0,
            end_ptr: 0,
        }
    }
}
