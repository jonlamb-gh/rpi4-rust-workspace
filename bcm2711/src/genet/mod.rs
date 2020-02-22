//! BCM54213PE Gigabit Ethernet Transceiver

// TODO - use typenum::U* on the consts for bounded-registers checked setters

pub mod ext;
pub mod hfb;
pub mod hfb_regs;
pub mod intrl2;
pub mod intrl2_0;
pub mod intrl2_1;
pub mod mdio;
pub mod rbuf;
pub mod rx_desc;
pub mod rx_dma;
pub mod rx_ring;
pub mod sys;
pub mod tx_desc;
pub mod tx_dma;
pub mod tx_ring;
pub mod umac;

pub use crate::genet::ext::EXT;
pub use crate::genet::hfb::HFB;
pub use crate::genet::hfb_regs::HFBREGS;
pub use crate::genet::intrl2_0::INTRL2_0;
pub use crate::genet::intrl2_1::INTRL2_1;
pub use crate::genet::mdio::MDIO;
pub use crate::genet::rbuf::RBUF;
pub use crate::genet::rx_dma::RXDMA;
pub use crate::genet::sys::SYS;
pub use crate::genet::tx_dma::TXDMA;
pub use crate::genet::umac::UMAC;

pub const PADDR: usize = 0xFD58_0000;

pub const SYS_PADDR: usize = PADDR + 0x0000;
pub const EXT_PADDR: usize = PADDR + 0x0080;
pub const INTRL2_0_PADDR: usize = PADDR + 0x0200;
pub const INTRL2_1_PADDR: usize = PADDR + 0x0240;
pub const RBUF_PADDR: usize = PADDR + 0x0300;
pub const UMAC_PADDR: usize = PADDR + 0x0800;
pub const HFB_PADDR: usize = PADDR + 0x8000;
pub const HFB_REGS_PADDR: usize = PADDR + 0xFC00;
pub const MDIO_PADDR: usize = PADDR + 0x0E14;

pub const RX_DMA_PADDR: usize = PADDR + RX_DMA_OFFSET;
pub const TX_DMA_PADDR: usize = PADDR + TX_DMA_OFFSET;

/// Rx DMA block offset from base `PADDR`
/// * registers for the 256 descriptors
/// * registers for the 17 rings
/// * DMA control/status registers
pub const RX_DMA_OFFSET: usize = 0x2000;

/// Tx DMA block offset from base `PADDR`
/// * registers for the 256 descriptors
/// * registers for the 17 rings
/// * DMA control/status registers
pub const TX_DMA_OFFSET: usize = 0x4000;

/// Number of DMA descriptor rings, same for Rx/Tx
pub const NUM_DMA_RINGS: usize = 17;

/// Max of 16 priority queues and 1 default queue
pub const DEFAULT_Q: usize = 16;

/// Each DMA descriptor is 3 words (12 bytes)
pub const DMA_DESC_WORDS: usize = 3;
pub const DMA_DESC_SIZE: usize = DMA_DESC_WORDS * 4;

/// Number of DMA descriptors, same for Rx/Tx
pub const NUM_DMA_DESC: usize = 256;

/// Buffer descriptors per Tx queue
pub const DMA_TX_BDS_PER_Q: usize = 32;
pub const DMA_TX_QUEUES: usize = 4;

/// DMA rings are 64 bytes
pub const DMA_RING_SIZE: usize = 0x40;

/// Total size of DMA rings
pub const DMA_RINGS_SIZE: usize = DMA_RING_SIZE * NUM_DMA_RINGS;

pub const DMA_MAX_BURST_LENGTH: usize = 8;

pub const RX_QUEUES: usize = 0;
pub const TX_QUEUES: usize = 4;

pub const RX_BDS_PER_Q: usize = 0;
pub const TX_BDS_PER_Q: usize = 32;

pub const Q16_RX_BD_CNT: usize = NUM_DMA_DESC - (RX_QUEUES * RX_BDS_PER_Q);
pub const Q16_TX_BD_CNT: usize = NUM_DMA_DESC - (TX_QUEUES * TX_BDS_PER_Q);

pub const DMA_FC_THRESH_HI: usize = NUM_DMA_DESC >> 4;
pub const DMA_FC_THRESH_LO: usize = 5;

// Highest priority queue
pub const Q0_PRIORITY: usize = 0;

pub const DMA_RING_BUF_PRIORITY_SHIFT: usize = 5;

// TODO - QTAG mask and shift overlap with other fields?
pub const QTAG_MASK: u32 = 0x3F;
pub const QTAG_SHIFT: usize = 7;
