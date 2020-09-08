//! Multispeed Internal RC oscillator clock.

use crate::periph::msi::MsiPeriph;
use crate::tasks::root::SystemRes;
use drone_cortexm::reg::prelude::*;

/// MSI driver.
pub struct Msi {
    periph: MsiPeriph,
}

impl Msi {
    /// Creates a new [`Msi`].
    #[inline]
    pub fn new(periph: MsiPeriph) -> Self {
        Self { periph }
    }

    /// Releases the peripheral.
    #[inline]
    pub fn free(self) -> MsiPeriph {
        self.periph
    }

    /// Initializes MSI.
    pub fn init(&self, res: &SystemRes) {
        self.periph.rcc_cr_msipllen.modify(|r| {
            self.periph.rcc_cr_msipllen.set(r);
            self.periph.rcc_cr_msirange.write(r, res.msirange);
            self.periph.rcc_cr_msirgsel.set(r);
        });
        while !self.periph.rcc_cr_msirdy.read_bit() {}
    }

    /// Reset MSI configuration to defaults.
    pub fn reset(&self) {
        self.periph.rcc_cr_msipllen.modify(|r| {
            self.periph.rcc_cr_msipllen.clear(r);
            self.periph.rcc_cr_msion.clear(r);
        });
    }

    /// Reads the MSIRANGE register field and returns it's value.
    pub fn read_msirange(&self) -> u32 {
        self.periph.rcc_cr_msirange.read_bits() as u32
    }
}
