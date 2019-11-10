use crate::eth::Eth;

impl Eth {
    pub(crate) fn intr_disable(&mut self) {
        // Mask all interrupts
        self.dev.intrl2_0.mask_set.write(0xFFFF_FFFF);
        self.dev.intrl2_0.clear.write(0xFFFF_FFFF);
        self.dev.intrl2_1.mask_set.write(0xFFFF_FFFF);
        self.dev.intrl2_1.clear.write(0xFFFF_FFFF);
    }
}
