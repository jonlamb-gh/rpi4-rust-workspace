//! I2C4

use crate::i2c0::RegisterBlock;
use crate::MMIO_BASE;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

pub const PADDR: usize = MMIO_BASE + 0x20_5800;

pub struct I2C4 {
    _marker: PhantomData<*const ()>,
}

unsafe impl Send for I2C4 {}

impl I2C4 {
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

impl Deref for I2C4 {
    type Target = RegisterBlock;
    fn deref(&self) -> &RegisterBlock {
        unsafe { &*self.as_ptr() }
    }
}

impl DerefMut for I2C4 {
    fn deref_mut(&mut self) -> &mut RegisterBlock {
        unsafe { &mut *self.as_mut_ptr() }
    }
}
