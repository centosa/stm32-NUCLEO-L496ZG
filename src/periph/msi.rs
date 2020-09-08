//! Multispeed Internal RC oscillator clock.

use drone_core::periph;

periph::singular! {
    /// Extracts MSI register tokens.
    pub macro periph_msi;

    /// MSI peripheral.
    pub struct MsiPeriph;

    drone_stm32_map::reg;
    crate::periph::msi;

    RCC {
        CR {
            MSIPLLEN;
            MSIRANGE;
            MSIRGSEL;
            MSION;
            MSIRDY;
        }
    }
}
