//! BCM54213PE Gigabit Ethernet driver
//!
//! Most of this implementation was based on work from the `rsta2/circle`
//! project: https://github.com/rsta2/circle

// https://github.com/rsta2/circle/blob/master/lib/bcm54213.cpp#L606
// https://github.com/torvalds/linux/blob/master/drivers/net/ethernet/broadcom/genet/bcmgenet.c

// TODO
// - checked fields
// - error handling
// - dropped/fragmented frames (remove the panics)
// - old discards handling logic in dma_recv
// - add log macro statements for debugging
// - dma/cache ops once caches are enabled
// - padding of user pkt in transmit path
//
// - tx isn't working yet
// - needs the InterruptHandler0, tx_reclaim() logic
//
//
// Issues
// - after a while, rx starts returing "Eth Error Fragmented"
//   * seems to do that for a while, then eventually get normal
//   ok frames
//   might just be some crap on my network?
//
// - sending l2 frames from my desktop, don't always get recv'd?

mod address;
mod control_block;
mod dma;
mod hfb;
mod intr;
mod mdio;
mod mii;
mod netif;
mod phy;
mod rx_ring;
mod tx_ring;
mod umac;

pub use crate::eth::address::EthernetAddress;
pub use crate::eth::control_block::ControlBlock;
pub use crate::eth::rx_ring::RxRing;
pub use crate::eth::tx_ring::TxRing;

use crate::timer::SysCounter;
use bcm2711::genet::umac::Cmd;
use bcm2711::genet::*;

const GENET_V5: u8 = 5;

/// Hw adds 2 bytes for IP alignment
const LEADING_PAD: usize = 2;

const FCS_LEN: usize = 4;

pub const MAX_MTU_SIZE: usize = 1536;
pub const MIN_MTU_SIZE: usize = 60;
pub const RX_BUF_LENGTH: usize = 2048;

/// Control blocks to manage the hw descriptors, same for Rx and Tx
pub type ControlBlocks = [ControlBlock; NUM_DMA_DESC];

pub type RxRings = [RxRing; NUM_DMA_RINGS];
pub type TxRings = [TxRing; NUM_DMA_RINGS];

pub struct Devices {
    pub sys: SYS,
    pub ext: EXT,
    pub intrl2_0: INTRL2_0,
    pub intrl2_1: INTRL2_1,
    pub rbuf: RBUF,
    pub umac: UMAC,
    pub hfb: HFB,
    pub hfb_regs: HFBREGS,
    pub mdio: MDIO,
    pub rdma: RXDMA,
    pub tdma: TXDMA,
}

impl Devices {
    pub fn new() -> Self {
        Devices {
            sys: SYS::new(),
            ext: EXT::new(),
            intrl2_0: INTRL2_0::new(),
            intrl2_1: INTRL2_1::new(),
            rbuf: RBUF::new(),
            umac: UMAC::new(),
            hfb: HFB::new(),
            hfb_regs: HFBREGS::new(),
            mdio: MDIO::new(),
            rdma: RXDMA::new(),
            tdma: TXDMA::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Error {
    HwVersionNotSupported,
    HwError,
    Fragmented,
    Malformed,
    Exhausted,
    Dropped,
}

pub struct Eth {
    dev: Devices,
    timer: SysCounter,
    crc_fwd_en: bool,

    link_status: bool,
    speed: u16,
    full_duplex: bool,
    pause: bool,
    // TODO - fix up this pattern
    old_link_status: Option<bool>,
    old_speed: Option<u16>,
    old_full_duplex: Option<bool>,
    old_pause: Option<bool>,

    rx_cbs: &'static mut [ControlBlock],
    tx_cbs: &'static mut [ControlBlock],
    rx_rings: &'static mut [RxRing],
    tx_rings: &'static mut [TxRing],
}

impl Eth {
    pub fn new(
        devices: Devices,
        timer: SysCounter,
        mac_address: EthernetAddress,
        rx_cbs: &'static mut [ControlBlock],
        tx_cbs: &'static mut [ControlBlock],
        rx_rings: &'static mut [RxRing],
        tx_rings: &'static mut [TxRing],
    ) -> Result<Self, Error> {
        let version_major = match devices
            .sys
            .rev_ctrl
            .get_field(sys::RevCtrl::Major::Read)
            .unwrap()
            .val()
        {
            6 => 5,
            _ => 0,
        };
        let version_minor: u8 = devices
            .sys
            .rev_ctrl
            .get_field(sys::RevCtrl::Minor::Read)
            .unwrap()
            .val() as _;
        let version_phy: u8 = devices
            .sys
            .rev_ctrl
            .get_field(sys::RevCtrl::EPhy::Read)
            .unwrap()
            .val() as _;

        if (version_major != GENET_V5) || (version_minor != 0) || (version_phy != 0) {
            return Err(Error::HwVersionNotSupported);
        }

        let mut eth = Eth {
            dev: devices,
            timer,
            crc_fwd_en: false,
            link_status: false,
            speed: 0,
            full_duplex: false,
            pause: false,
            old_link_status: None,
            old_speed: None,
            old_full_duplex: None,
            old_pause: None,
            rx_cbs,
            tx_cbs,
            rx_rings,
            tx_rings,
        };

        eth.umac_reset();
        eth.umac_reset2();
        eth.umac_init();

        // Make sure we reflect the value of CRC_CMD_FWD
        eth.crc_fwd_en = eth.dev.umac.cmd.is_set(Cmd::CrcFwd::Read);

        eth.umac_set_hw_addr(&mac_address);

        // Disable Rx/Tx DMA and flush Tx queues
        eth.dma_disable();

        // Reinitialize TxDMA and RxDMA
        eth.dma_init();

        // Always enable ring 16 - descriptor ring
        eth.dma_enable();

        eth.hfb_init();

        // TODO
        // interrupts

        eth.mii_probe();

        eth.netif_start();

        eth.umac_set_rx_mode(&mac_address);

        Ok(eth)
    }

    pub fn link_up(&self) -> bool {
        self.link_status
    }

    pub fn link_speed(&self) -> u16 {
        self.speed
    }

    // In circle this is called every 2 seconds
    pub fn update_phy(&mut self) {
        self.phy_read_status();
        self.mii_setup();
    }

    pub fn recv(&mut self, pkt: &mut [u8]) -> Result<usize, Error> {
        self.dma_recv(pkt)
    }

    pub fn send(&mut self, pkt: &[u8]) -> Result<(), Error> {
        self.dma_send(pkt)
    }
}
