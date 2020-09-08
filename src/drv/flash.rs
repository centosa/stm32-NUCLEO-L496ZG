//! Embedded Flash memory.

use crate::periph::flash::FlashPeriph;
use drone_cortexm::reg::prelude::*;

/// Flash driver.
pub struct Flash {
    periph: FlashPeriph,
}

impl Flash {
    /// Creates a new [`Flash`].
    #[inline]
    pub fn new(periph: FlashPeriph) -> Self {
        Self { periph }
    }

    /// Releases the peripheral.
    #[inline]
    pub fn free(self) -> FlashPeriph {
        self.periph
    }

    /// Initializes flash.
    pub fn init(&self) {
        self.periph
            .flash_acr
            .store(|r| r.set_prften().set_icen().set_dcen().write_latency(5));
    }

    /// Set the read access latency for flash.
    pub fn set_latency(&self, latency: u32) {
        println!("Set latency to {}", latency);
        self.periph
            .flash_acr
            .store(|r| r.set_prften().set_icen().set_dcen().write_latency(latency));
    }
}
