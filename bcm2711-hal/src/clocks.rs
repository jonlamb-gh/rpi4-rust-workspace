//! Clocks

use crate::mailbox::{Channel, Error, GetClockRateRepr, Mailbox, RespMsg};
use crate::time::Hertz;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ClockId {
    Emmc,
    Uart,
    Arm,
    Core,
    Unkown(u32),
}

impl From<ClockId> for u32 {
    fn from(id: ClockId) -> u32 {
        match id {
            ClockId::Emmc => 0x01,
            ClockId::Uart => 0x02,
            ClockId::Arm => 0x03,
            ClockId::Core => 0x04,
            ClockId::Unkown(id) => id,
        }
    }
}

impl From<u32> for ClockId {
    fn from(id: u32) -> ClockId {
        match id {
            0x01 => ClockId::Emmc,
            0x02 => ClockId::Uart,
            0x03 => ClockId::Arm,
            0x04 => ClockId::Core,
            _ => ClockId::Unkown(id),
        }
    }
}

/// Frozen clock frequencies
///
/// The existence of this value indicates that the clock configuration can no
/// longer be changed
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Clocks {
    core: Hertz,
    arm: Hertz,
    uart: Hertz,
}

impl Clocks {
    pub fn freeze(mbox: &mut Mailbox) -> Result<Self, Error> {
        let resp = mbox.call(Channel::Prop, &GetClockRateRepr::new(ClockId::Core.into()))?;
        let core = if let RespMsg::GetClockRate(repr) = resp {
            repr
        } else {
            return Err(Error::Malformed);
        };

        let resp = mbox.call(Channel::Prop, &GetClockRateRepr::new(ClockId::Arm.into()))?;
        let arm = if let RespMsg::GetClockRate(repr) = resp {
            repr
        } else {
            return Err(Error::Malformed);
        };

        let resp = mbox.call(Channel::Prop, &GetClockRateRepr::new(ClockId::Uart.into()))?;
        let uart = if let RespMsg::GetClockRate(repr) = resp {
            repr
        } else {
            return Err(Error::Malformed);
        };

        Ok(Clocks {
            core: core.rate(),
            arm: arm.rate(),
            uart: uart.rate(),
        })
    }

    /// Returns the frequency of the core clock
    pub fn core(&self) -> Hertz {
        self.core
    }

    /// Returns the frequency of the ARM clock
    pub fn arm(&self) -> Hertz {
        self.arm
    }

    /// Returns the frequency of the UART clock
    pub fn uart(&self) -> Hertz {
        self.uart
    }
}
