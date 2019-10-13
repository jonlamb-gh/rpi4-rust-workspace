// INTRL2_0 and INTRL2_1 have the same registers, each is a bitfield defined
// here
use bitfield::bitfield;

bitfield! {
    #[repr(C)]
    pub struct INTRL2(u32);
    impl Debug;
    u32;
    pub scb, set_scb : 0;
    pub ephy, set_ephy : 1;
    pub phy_det_r, set_phy_det_r : 2;
    pub phy_det_f, set_phy_det_f : 3;
    pub link_up, set_link_up : 4;
    pub link_down, set_link_down : 5;
    pub umac, set_umac : 6;
    pub umac_tsv, set_umac_tsv : 7;
    pub tbuf_underrun, set_tbuf_underrun : 8;
    pub rbuf_overflow, set_rbuf_overflow : 9;
    pub hfb_sm, set_hfb_sm : 10;
    pub hfb_mm, set_hfb_mm : 11;
    pub mpd_r, set_mpd_r : 12;
    pub rxdma_mbdone, set_rxdma_mbdone : 13;
    pub rxdma_pdone, set_rxdma_pdone : 14;
    pub rxdma_bdone, set_rxdma_bdone : 15;
    pub txdma_mbdone, set_txdma_mbdone : 16;
    pub txdma_pdone, set_txdma_pdone : 17;
    pub txdma_bdone, set_txdma_bdone : 18;
    // v3+
    pub mdio_done, set_mdio_done : 23;
    pub mdio_error, set_mdio_error : 24;
}
