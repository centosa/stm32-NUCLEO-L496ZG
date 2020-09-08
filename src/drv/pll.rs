//! Phase-Locked Loop clock.

use crate::periph::pll::PllPeriph;
use crate::tasks::root::SystemRes;
use drone_cortexm::reg::prelude::*;

/// PLL driver.
pub struct Pll {
    periph: PllPeriph,
}

impl Pll {
    /// Creates a new [`Pll`].
    #[inline]
    pub fn new(periph: PllPeriph) -> Self {
        Self { periph }
    }

    /// Releases the peripheral.
    #[inline]
    pub fn free(self) -> PllPeriph {
        self.periph
    }

    /// Initializes PLL.
    pub fn init(&self, res: &SystemRes) {
        self.periph.rcc_pllcfgr.store(|r| {
            r.write_pllsrc(res.pllsrc)
                .write_pllm(res.pllm - 1)
                .write_plln(res.plln)
                .write_pllr((res.pllr >> 1) - 1)
                .set_pllren()
        });
    }

    /// Enables PLL.
    pub fn enable(&self) {
        println!("PLL enable");
        self.periph.rcc_cr_pllon.set_bit();
        while !self.periph.rcc_cr_pllrdy.read_bit() {}
        println!("PLL is enabled");
    }

    /// Disable PLL.
    pub fn disable(&self) {
        self.periph.rcc_cr_pllon.clear_bit();
        while self.periph.rcc_cr_pllrdy.read_bit() {}
    }

    /// Resets the PLL configuration to default.
    #[inline]
    pub fn reset(&self) {
        self.periph.rcc_cr_pllon.clear_bit();
        self.periph.rcc_pllcfgr.reset();
        while self.periph.rcc_cr_pllrdy.read_bit() {}
    }

    /// Returns value of field PLLSRC.
    #[inline]
    pub fn read_pllsrc(&self) -> u32 {
        self.periph.rcc_pllcfgr.pllsrc.read_bits() as u32
    }

    /// Returns value of field PLLN.
    #[inline]
    pub fn read_plln(&self) -> u32 {
        self.periph.rcc_pllcfgr.plln.read_bits() as u32
    }

    /// Returns value of field PLLN.
    #[inline]
    pub fn read_pllr(&self) -> u32 {
        self.periph.rcc_pllcfgr.pllr.read_bits() as u32
    }
}
