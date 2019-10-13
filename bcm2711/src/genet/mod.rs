//! BCM54213PE Gigabit Ethernet Transceiver

pub mod ext;
pub mod hfb;
pub mod hfb_regs;
pub mod intrl2;
pub mod intrl2_0;
pub mod intrl2_1;
pub mod mdio;
pub mod rbuf;
pub mod rx_dma;
pub mod sys;
pub mod umac;

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

/// Max of 16 priority queues and 1 default queue
pub const DESC_INDEX: usize = 16;

/// Each DMA descriptor is 3 words (12 bytes)
pub const DMA_DESC_WORDS: usize = 3;
pub const DMA_DESC_SIZE: usize = DMA_DESC_WORDS * 4;

/// Number of DMA descriptors, same for Rx/Tx
pub const NUM_DMA_DESC: usize = 256;

/// DMA ring size
pub const DMA_RING_SIZE: usize = 0x40;

/// Total size of DMA rings
pub const DMA_RINGS_SIZE: usize = DMA_RING_SIZE * (DESC_INDEX + 1);

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

/// Rx DMA registers offset from base `PADDR`
pub const RX_DMA_REGS_OFFSET: usize = RX_DMA_OFFSET + (NUM_DMA_DESC * DMA_DESC_SIZE);

/// Tx DMA registers offset from base `PADDR`
pub const TX_DMA_REGS_OFFSET: usize = TX_DMA_OFFSET + (NUM_DMA_DESC * DMA_DESC_SIZE);
