#![no_std]

#[macro_use]
extern crate bounded_registers;
#[macro_use]
extern crate typenum;

const MMIO_BASE: usize = 0xFE00_0000;

pub mod dma;
pub mod genet;
pub mod gpio;
pub mod i2c0;
pub mod i2c1;
pub mod i2c2;
pub mod i2c3;
pub mod i2c4;
pub mod i2c5;
pub mod i2c6;
pub mod mbox;
pub mod rng;
pub mod spi0;
pub mod spi1;
pub mod spi2;
pub mod sys_timer;
pub mod uart0;
pub mod uart1;
