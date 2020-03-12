#![no_std]

extern crate bcm2711_hal as hal;

use crate::hal::bcm2711::genet::NUM_DMA_DESC;
use crate::hal::eth::{Eth, RxPacket, MAX_MTU_SIZE};
use core::intrinsics::transmute;
use core::ops::DerefMut;
use smoltcp::phy::{self, Device, DeviceCapabilities};
use smoltcp::time::Instant;
use smoltcp::{Error, Result};

pub struct EthDevice<'rx, 'tx> {
    pub eth: Eth<'rx, 'tx>,
}

impl<'a, 'rx, 'tx> Device<'a> for EthDevice<'rx, 'tx> {
    type RxToken = RxToken<'a>;
    type TxToken = TxToken<'a>;

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = MAX_MTU_SIZE;
        caps.max_burst_size = Some(NUM_DMA_DESC / 2);
        caps
    }

    fn receive(&mut self) -> Option<(Self::RxToken, Self::TxToken)> {
        let self_ = unsafe {
            // HACK: eliminate lifetimes
            transmute::<&mut EthDevice<'rx, 'tx>, &mut EthDevice<'a, 'a>>(&mut *self)
        };

        let dev = self_ as *mut EthDevice<'a, 'a>;
        match self_.eth.recv() {
            Ok(pkt) if pkt.len() > 0 => {
                let rx = RxToken { pkt };
                let tx = TxToken { dev };
                Some((rx, tx))
            }
            Ok(_) => None,
            Err(_) => None,
        }
    }

    fn transmit(&mut self) -> Option<Self::TxToken> {
        let dev = unsafe {
            transmute::<&mut EthDevice<'rx, 'tx>, &mut EthDevice<'a, 'a>>(&mut *self)
                as *mut EthDevice<'a, 'a>
        };
        Some(TxToken { dev })
    }
}

pub struct RxToken<'a> {
    pkt: RxPacket<'a>,
}

impl<'a> phy::RxToken for RxToken<'a> {
    fn consume<R, F>(mut self, _timestamp: Instant, f: F) -> Result<R>
    where
        F: FnOnce(&mut [u8]) -> Result<R>,
    {
        let result = f(self.pkt.deref_mut());
        result
    }
}

pub struct TxToken<'a> {
    dev: *mut EthDevice<'a, 'a>,
}

impl<'a> phy::TxToken for TxToken<'a> {
    fn consume<R, F>(self, _timestamp: Instant, len: usize, f: F) -> Result<R>
    where
        F: FnOnce(&mut [u8]) -> Result<R>,
    {
        let dev = unsafe { &mut *self.dev };
        match dev.eth.send(len, f) {
            Err(_) => Err(Error::Exhausted),
            Ok(r) => r,
        }
    }
}
