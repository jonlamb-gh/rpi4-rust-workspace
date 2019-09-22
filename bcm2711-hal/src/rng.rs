//! RNG

use crate::hal::blocking::rng::Read;
use bcm2711::rng::*;
use cortex_a::asm;
use void::Void;

// TODO - replace with Freq/Hertz
const RNG_WARMUP_COUNT: u32 = 0x0004_0000;

/// RNG abstraction
pub struct Rng {
    rng: RNG,
}

impl Rng {
    pub fn new(rng: RNG) -> Self {
        // Disable ints
        rng.INT_MASK.modify(INT_MASK::INT_OFF::True);

        // Set warm-up count and enable
        rng.STATUS.modify(STATUS::COUNT.val(RNG_WARMUP_COUNT));
        rng.CTRL.modify(CTRL::ENABLE::True);

        Rng { rng }
    }

    pub fn free(self) -> RNG {
        self.rng.CTRL.modify(CTRL::ENABLE::False);
        self.rng
    }

    pub fn next_u32(&mut self) -> Result<u32, Void> {
        // Wait for entropy
        while self.rng.STATUS.is_set(STATUS::READY) == false {
            asm::nop();
        }

        Ok(self.rng.DATA.get())
    }
}

impl Read for Rng {
    type Error = Void;

    fn read(&mut self, buffer: &mut [u8]) -> Result<(), Self::Error> {
        let mut word: u32 = self.next_u32()?;

        // Fill buffer with rng words
        for (i, slot) in buffer.iter_mut().enumerate() {
            let byte_index = i % 4;

            if byte_index == 0 {
                word = self.next_u32()?;
            }

            *slot = (0xFF & (word >> (byte_index * 8))) as u8;
        }

        Ok(())
    }
}
