//! 16MHz internal RC oscillator clock.

use crate::periph::hsi16::Hsi16Periph;
use crate::tasks::root::SystemRes;
use drone_cortexm::reg::prelude::*;

/// HSI16 driver.
pub struct Hsi16 {
    periph: Hsi16Periph,
}

impl Hsi16 {
    /// Creates a new [`Hsi16`].
    #[inline]
    pub fn new(periph: Hsi16Periph) -> Self {
        Self { periph }
    }

    /// Releases the peripheral.
    #[inline]
    pub fn free(self) -> Hsi16Periph {
        self.periph
    }

    /// Initializes HSI16.
    pub fn init(&self, _res: &SystemRes) {
        println!("HSI16 init");
        self.periph.rcc_cr_hsion.modify(|r| {
            self.periph.rcc_cr_hsion.set(r);
        });
        while !self.periph.rcc_cr_hsirdy.read_bit_band() {}
    }

    /// Reset the HSI16 configuration to default
    pub fn reset(&self) {}
}
