//! Reset and Clock Control.

use crate::periph::rcc::RccPeriph;
use crate::tasks::root::SystemRes;
use drone_cortexm::reg::prelude::*;

/// RCC driver.
pub struct Rcc {
    periph: RccPeriph,
}

impl Rcc {
    /// Creates a new [`Rcc`].
    #[inline]
    pub fn new(periph: RccPeriph) -> Self {
        Self { periph }
    }

    /// Releases the peripheral.
    #[inline]
    pub fn free(self) -> RccPeriph {
        self.periph
    }

    /// Initializes RCC.
    #[inline]
    pub fn init(&self, res: &SystemRes) {
        self.periph
            .rcc_cfgr
            .store(|r| r.write_sw(res.clksrc).write_ppre1(0b110));
    }

    /// Reset RCC to default.
    pub fn reset(&self) {
        self.periph
            .rcc_cfgr
            .store(|r| r.write_sw(0b00).write_ppre1(0b110));
        self.periph.rcc_apb1enr1.reset();
    }

    /// Read the system clock switch status from mcu.
    pub fn read_sws(&self) -> u32 {
        self.periph.rcc_cfgr.sws.read_bits() as u32
    }

    /// Power interface clock enable.
    #[inline]
    pub fn set_apb1enr1_pwren(&self) -> () {
        self.periph.rcc_apb1enr1.modify(|r| r.set_pwren());
    }

    /// Power interface clock disable.
    #[inline]
    pub fn clear_apb1enr1_pwren(&self) -> () {
        self.periph.rcc_apb1enr1.modify(|r| r.clear_pwren());
    }

    /// Disable backup domain write protection
    #[inline]
    pub fn set_pwr_cr1_dbp(&self) {
        self.periph.pwr_cr1_dbp.set_bit();
    }

    /// Low-power mode selection.
    #[inline]
    pub fn write_pwr_cr1_lpms(&self, lpms: u32) -> () {
        self.periph.pwr_cr1_lpms.write_bits(lpms);
    }

    /// V-BAT battery charging resistor selection
    #[inline]
    pub fn set_pwr_cr4_vbrs(&self) -> () {
        self.periph.pwr_cr4_vbrs.set_bit();
    }

    /// V-BAT battery charging enable
    #[inline]
    pub fn set_pwr_cr4_vbe(&self) -> () {
        self.periph.pwr_cr4_vbe.set_bit();
    }
}
