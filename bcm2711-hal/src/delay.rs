//! Delays

use crate::clocks::Clocks;
use crate::hal::blocking::delay::{DelayMs, DelayUs};
use core::cmp;
use cortex_a::asm;

/// NOP used as a delay provider
/// NOTE: this is not accurate, for accurate timing and delays consider
/// using one of the timers
pub fn delay_us(us: u32, clocks: &Clocks) {
    // TODO - fix this
    let cnt = us * cmp::max(clocks.core().0 / 1_000_000, 1);
    for _ in 0..cnt {
        asm::nop();
    }
}

pub struct Delay {
    clocks: Clocks,
}

impl Delay {
    pub fn new(clocks: Clocks) -> Self {
        Delay { clocks }
    }
}

impl DelayMs<u32> for Delay {
    fn delay_ms(&mut self, ms: u32) {
        self.delay_us(ms * 1_000);
    }
}

impl DelayMs<u16> for Delay {
    fn delay_ms(&mut self, ms: u16) {
        self.delay_ms(ms as u32);
    }
}

impl DelayMs<u8> for Delay {
    fn delay_ms(&mut self, ms: u8) {
        self.delay_ms(ms as u32);
    }
}

impl DelayUs<u32> for Delay {
    fn delay_us(&mut self, us: u32) {
        delay_us(us, &self.clocks);
    }
}

impl DelayUs<u16> for Delay {
    fn delay_us(&mut self, us: u16) {
        self.delay_us(us as u32)
    }
}

impl DelayUs<u8> for Delay {
    fn delay_us(&mut self, us: u8) {
        self.delay_us(us as u32)
    }
}
