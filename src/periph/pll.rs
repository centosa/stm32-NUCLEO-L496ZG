//! Phase-Locked Loop clock.

use drone_core::periph;

periph::singular! {
    /// Extracts PLL register tokens.
    pub macro periph_pll;

    /// PLL peripheral.
    pub struct PllPeriph;

    drone_stm32_map::reg;
    crate::periph::pll;

    RCC {
        CR {
            PLLON;
            PLLRDY;
        }
        PLLCFGR;
    }
}

periph::singular! {
    /// Extracts PLLSAI1 register tokens.
    pub macro periph_pllsai1;

    /// PLLSAI1 peripheral.
    pub struct Pllsai1Periph;

    drone_stm32_map::reg;
    crate::periph::pll;

    RCC {
        CICR {
            PLLSAI1RDYC;
        }
        CIER {
            PLLSAI1RDYIE;
        }
        CIFR {
            PLLSAI1RDYF;
        }
        CR {
            PLLSAI1ON;
        }
        PLLSAI1CFGR;
    }
}
