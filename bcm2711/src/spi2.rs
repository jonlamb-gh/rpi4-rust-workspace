//! SPI2

use crate::spi0::RegisterBlock;
use crate::MMIO_BASE;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

pub const PADDR: usize = MMIO_BASE + 0x21_50C0;

pub struct SPI2 {
    _marker: PhantomData<*const ()>,
}

unsafe impl Send for SPI2 {}

impl SPI2 {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    pub fn as_ptr(&self) -> *const RegisterBlock {
        PADDR as *const _
    }

    pub fn as_mut_ptr(&mut self) -> *mut RegisterBlock {
        PADDR as *mut _
    }
}

impl Deref for SPI2 {
    type Target = RegisterBlock;
    fn deref(&self) -> &RegisterBlock {
        unsafe { &*self.as_ptr() }
    }
}

impl DerefMut for SPI2 {
    fn deref_mut(&mut self) -> &mut RegisterBlock {
        unsafe { &mut *self.as_mut_ptr() }
    }
}
