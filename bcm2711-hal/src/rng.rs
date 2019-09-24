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
    pub fn new(mut rng: RNG) -> Self {
        // Disable ints
        rng.int_mask.modify(IntMask::IntOff::Set);

        // Set warm-up count and enable
        rng.status
            .modify(Status::Count::Field::new(RNG_WARMUP_COUNT).unwrap());
        rng.control.modify(Control::Enable::Set);

        Rng { rng }
    }

    pub fn free(mut self) -> RNG {
        self.rng.control.modify(Control::Enable::Clear);
        self.rng
    }

    pub fn next_u32(&mut self) -> Result<u32, Void> {
        // Wait for entropy
        while self.rng.status.is_set(Status::Ready::Read) == false {
            asm::nop();
        }

        Ok(self.rng.data.get_field(Fifo::Data::Read).unwrap().val())
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
