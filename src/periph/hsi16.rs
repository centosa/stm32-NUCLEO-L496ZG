//! 16MHz Internal RC oscillator clock.

use drone_core::periph;

periph::singular! {
    /// Extracts HSI16 register tokens.
    pub macro periph_hsi16;

    /// HSI16 peripheral.
    pub struct Hsi16Periph;

    drone_stm32_map::reg;
    crate::periph::hsi16;

    RCC {
        CR {
            HSION;
            HSIRDY;
        }
    }
}
