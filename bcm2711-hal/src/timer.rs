//! Timer abstractions over the System Timer
//!
//! TODO - update this once bcm2711 docs are available
//!
//! The System Timer peripheral provides four 32-bit timer
//! channels and a single 64-bit free running counter.
//!
//! They run at 1 MHz, each count/tick represents a microsecond.
//!
//! Note that the GPU uses timer channels 0 and 2, so this
//! interface only exposes channels 1 and 3.

// TODO
// Events, IRQ must be enabled in the int controller

use crate::hal::blocking::delay::{DelayMs, DelayUs};
use crate::hal::timer::{CountDown, Periodic};
use crate::time::{Hertz, Instant};
use bcm2711::sys_timer::*;
use core::convert::TryFrom;
use void::Void;

const CLOCK_FREQ: Hertz = Hertz(1_000_000);

/// System counter
///
/// Abstraction over the read-only 64-bit free running counter.
pub struct SysCounter {
    timer: SysTimer,
}

/// Hardware timer
///
/// Abstraction over one of the four 32-bit timer channels.
pub struct Timer {
    channel: Channel,
    timer: SysTimer,
    ticks: u32,
}

enum Channel {
    C1,
    C3,
}

/// Extension trait to split a SysTimer peripheral in to independent timers and
/// registers
pub trait TimerExt {
    /// The parts to split the SysTimer into
    type Parts;

    /// Splits the SysTimer block into independent parts
    fn split(self) -> Self::Parts;
}

pub struct Parts {
    pub sys_counter: SysCounter,
    pub timer1: Timer,
    pub timer3: Timer,
}

impl TimerExt for SysTimer {
    type Parts = Parts;

    fn split(self) -> Self::Parts {
        // Each timer/counter gets a copy of the base SysTimer paddr/device
        Parts {
            sys_counter: SysCounter {
                timer: SysTimer::new(),
            },
            timer1: Timer {
                timer: SysTimer::new(),
                channel: Channel::C1,
                ticks: 0,
            },
            timer3: Timer {
                timer: SysTimer::new(),
                channel: Channel::C3,
                ticks: 0,
            },
        }
    }
}

// TODO - use macro
impl Timer {
    fn t1_update(&mut self) {
        // Clear the interrupt, if any, write a 1 to clear
        self.timer.CS.modify(CS::MATCH1::SET);

        // Timers compare against the lower 32 bits of the u64 counter
        let cmp = self.timer.LO.get().wrapping_add(self.ticks);

        // Set the output compare register
        self.timer.C0.set(cmp);
    }

    fn t1_wait(&mut self) -> nb::Result<(), Void> {
        if self.timer.CS.is_set(CS::MATCH1) {
            // Ack/clear and update compare
            self.t1_update();
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    fn t3_update(&mut self) {
        // Clear the interrupt, if any, write a 1 to clear
        self.timer.CS.modify(CS::MATCH3::SET);

        // Timers compare against the lower 32 bits of the u64 counter
        let cmp = self.timer.LO.get().wrapping_add(self.ticks);

        // Set the output compare register
        self.timer.C3.set(cmp);
    }

    fn t3_wait(&mut self) -> nb::Result<(), Void> {
        if self.timer.CS.is_set(CS::MATCH3) {
            // Ack/clear and update compare
            self.t3_update();
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl Periodic for Timer {}

impl CountDown for Timer {
    type Time = Hertz;

    fn start<T>(&mut self, timeout: T)
    where
        T: Into<Hertz>,
    {
        let freq = timeout.into();

        self.ticks = CLOCK_FREQ.0 / freq.0;

        match self.channel {
            Channel::C1 => self.t1_update(),
            Channel::C3 => self.t3_update(),
        }
    }

    fn wait(&mut self) -> nb::Result<(), Void> {
        match self.channel {
            Channel::C1 => self.t1_wait(),
            Channel::C3 => self.t3_wait(),
        }
    }
}

impl SysCounter {
    pub fn get_time(&self) -> Instant {
        // Microseconds to millisconds
        Instant::from_millis(i64::try_from(self.read()).unwrap() / 1000)
    }

    pub fn read(&self) -> u64 {
        read_sys_counter(&self.timer)
    }

    pub fn delay_us_from_now(&self, us: u32) {
        let now = self.read();
        let end = now.wrapping_add(u64::from(us));

        loop {
            if self.read() > end {
                break;
            }
        }
    }
}

fn read_sys_counter(timer: &SysTimer) -> u64 {
    let mut hi = timer.HI.get();
    let mut lo = timer.LO.get();

    // We have to repeat if high word changed during read
    if hi != timer.HI.get() {
        hi = timer.HI.get();
        lo = timer.LO.get();
    }

    (u64::from(hi) << 32) | u64::from(lo)
}

impl DelayMs<u32> for SysCounter {
    fn delay_ms(&mut self, ms: u32) {
        self.delay_us(ms * 1_000);
    }
}

impl DelayMs<u16> for SysCounter {
    fn delay_ms(&mut self, ms: u16) {
        self.delay_ms(ms as u32);
    }
}

impl DelayMs<u8> for SysCounter {
    fn delay_ms(&mut self, ms: u8) {
        self.delay_ms(ms as u32);
    }
}

impl DelayUs<u32> for SysCounter {
    fn delay_us(&mut self, us: u32) {
        self.delay_us_from_now(us);
    }
}

impl DelayUs<u16> for SysCounter {
    fn delay_us(&mut self, us: u16) {
        self.delay_us(us as u32)
    }
}

impl DelayUs<u8> for SysCounter {
    fn delay_us(&mut self, us: u8) {
        self.delay_us(us as u32)
    }
}
