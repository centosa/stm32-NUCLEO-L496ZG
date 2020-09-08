//! Reset and Clock Control.

use drone_core::periph;

periph::singular! {
    /// Extracts RCC register tokens.
    pub macro periph_rcc;

    /// RCC peripheral.
    pub struct RccPeriph;

    drone_stm32_map::reg;
    crate::periph::rcc;

    RCC {
        CFGR;
        APB1ENR1;
    }

    PWR {
        CR1 {
            DBP;
            LPMS;
        }
        CR4 {
            VBRS;
            VBE;
        }
    }

}
