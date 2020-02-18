//! BCM54213PE Gigabit Ethernet driver
//!
//! This implementation was based on:
//! https://github.com/u-boot/u-boot/blob/master/drivers/net/bcmgenet.c

// https://github.com/u-boot/u-boot/blob/master/drivers/net/bcmgenet.c
//
// functions:
// * bcmgenet_umac_reset
// * bcmgenet_disable_dma
// * bcmgenet_enable_dma
// * invalidate_dcache_check
// * bcmgenet_adjust_link
// * bcmgenet_phy_init
// * bcmgenet_interface_set
// * bcmgenet_eth_probe
// * bcmgenet_eth_ofdata_to_platdata
// * rx_descs_init
// * rx_ring_init
// * tx_ring_init
//
// * bcmgenet_gmac_write_hwaddr
// * bcmgenet_gmac_eth_send
// * bcmgenet_gmac_eth_recv
// * bcmgenet_gmac_free_pkt
// * bcmgenet_gmac_eth_start
// * bcmgenet_gmac_eth_stop
//
// * bcmgenet_mdio_start
// * bcmgenet_mdio_write
// * bcmgenet_mdio_read
// * bcmgenet_mdio_init
//
// structs:
// * bcmgenet_eth_priv
//   - holds an rx buffer, to be split up for the DMA ops
// * bcmgenet_gmac_eth_ops
// * bcmgenet_eth_ids

mod address;
mod descriptor;
mod dma;
mod mdio;
mod mii;
mod netif;
mod phy;
mod umac;

//mod control_block;
//mod hfb;
//mod intr;
//mod rx_ring;
//mod tx_ring;

pub use crate::eth::address::EthernetAddress;
pub use crate::eth::descriptor::Descriptor;
pub use crate::eth::phy::Status as PhyStatus;

//pub use crate::eth::control_block::ControlBlock;
//pub use crate::eth::rx_ring::RxRing;
//pub use crate::eth::tx_ring::TxRing;

use crate::timer::SysCounter;
use bcm2711::genet::*;

const GENET_V5: u8 = 5;

// Hw adds 2 bytes for IP alignment
// == RX_BUF_OFFSET
const LEADING_PAD: usize = 2;

const FCS_LEN: usize = 4;

// Body(1500) + EH_SIZE(14) + VLANTAG(4) + BRCMTAG(6) + FCS(4) = 1528.
// 1536 is multiple of 256 bytes
pub const MAX_MTU_SIZE: usize = 1536;
//pub const MAX_MTU_SIZE: usize =
//    ETH_DATA_LEN + ETH_HLEN + VLAN_HLEN + ENET_BRCM_TAG_LEN + ETH_FCS_LEN +
// ENET_PAD;

pub const MIN_MTU_SIZE: usize = 60;
pub const RX_BUF_LENGTH: usize = 2048;
pub const TX_BUF_LENGTH: usize = 2048;

pub type Descriptors = [Descriptor; NUM_DMA_DESC];

//pub type RxRings = [RxRing; NUM_DMA_RINGS];
//pub type TxRings = [TxRing; NUM_DMA_RINGS];

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
    TimedOut,
}

pub struct Eth {
    c_index: usize,
    rx_index: usize,
    tx_index: usize,

    dev: Devices,
    // TODO - borrow the timer instead
    timer: SysCounter,
    // TODO - use refs or storage instead
    rx_mem: &'static mut [Descriptor],
    /*rx_cbs: &'static mut [ControlBlock],
     *tx_cbs: &'static mut [ControlBlock],
     *rx_rings: &'static mut [RxRing],
     *tx_rings: &'static mut [TxRing], */
}

impl Eth {
    pub fn new(
        devices: Devices,
        timer: SysCounter,
        mac_address: EthernetAddress,
        rx_mem: &'static mut [Descriptor],
        /*rx_cbs: &'static mut [ControlBlock],
         *tx_cbs: &'static mut [ControlBlock],
         *rx_rings: &'static mut [RxRing],
         *tx_rings: &'static mut [TxRing], */
    ) -> Result<Self, Error> {
        assert_eq!(rx_mem.len(), NUM_DMA_DESC);

        // TODO https://github.com/u-boot/u-boot/blob/master/drivers/net/bcmgenet.c#L626
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
            c_index: 0,
            rx_index: 0,
            tx_index: 0,
            dev: devices,
            timer,
            rx_mem,
            /*rx_cbs,
             *tx_cbs,
             *rx_rings,
             *tx_rings, */
        };

        //
        // stuff from bcmgenet_eth_probe()

        eth.mii_config();

        eth.umac_reset();

        // bcmgenet_mdio_init()
        // TODO - need to call mdio_reset()?

        // bcmgenet_phy_init()
        // TODO - anything here, phy stuff?

        //
        // now things from bcmgenet_gmac_eth_start()
        //

        // bcmgenet_umac_reset()
        eth.umac_reset2();
        eth.umac_reset();
        eth.umac_init();

        // bcmgenet_gmac_write_hwaddr()
        eth.umac_set_hw_addr(&mac_address);

        // Disable RX/TX DMA and flush TX queues
        // bcmgenet_disable_dma()
        eth.dma_disable();

        eth.rx_ring_init();
        eth.rx_descs_init();
        eth.tx_ring_init();

        // Enable RX/TX DMA
        // bcmgenet_enable_dma(priv);
        eth.dma_enable();

        // read PHY properties over the wire from generic PHY set-up
        // phy_startup()
        // TODO
        let status = eth.phy_read_status();
        assert_eq!(status.link_status, true, "Link is down");
        assert_ne!(status.speed, 0, "Speed is 0");
        assert_eq!(status.full_duplex, true, "Not full duplex");

        // Update MAC registers based on PHY property
        // bcmgenet_adjust_link()
        eth.mii_setup(&status);

        // Enable Rx/Tx
        eth.netif_start();

        Ok(eth)
    }

    pub fn recv(&mut self, pkt: &mut [u8]) -> Result<usize, Error> {
        self.dma_recv(pkt)
    }

    pub fn send(&mut self, pkt: &[u8]) -> Result<(), Error> {
        self.dma_send(pkt)
    }
}
