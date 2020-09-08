//! GPIO pins bindings.

use crate::drv::gpio::GpioHeadEn;

use drone_core::inventory;
use drone_cortexm::reg::prelude::*;
use drone_stm32_map::periph::gpio::{
    head::{GpioBHead, GpioCHead},
    pin::{GpioB14, GpioB7, GpioC13, GpioC7, GpioPinPeriph},
};

/// Acquires [`GpioPins`].
#[doc(hidden)]
#[macro_export]
macro_rules! drv_gpio_pins {
    ($reg:ident) => {
        $crate::sys::gpio_pins::GpioPins::new($crate::sys::gpio_pins::GpioPinsRes {
            gpio_b7: ::drone_stm32_map::periph::gpio::periph_gpio_b7!($reg),
            gpio_b14: ::drone_stm32_map::periph::gpio::periph_gpio_b14!($reg),
            gpio_c7: ::drone_stm32_map::periph::gpio::periph_gpio_c7!($reg),
            gpio_c13: ::drone_stm32_map::periph::gpio::periph_gpio_c13!($reg),
        })
    };
}

/// GPIO pins driver.
pub struct GpioPins(GpioPinsRes);

/// GPIO pins resource for driving the LEDs on the NUCLEO.
pub struct GpioPinsRes {
    /// LD1.
    pub gpio_c7: GpioPinPeriph<GpioC7>,
    /// LD2.
    pub gpio_b7: GpioPinPeriph<GpioB7>,
    /// LD3.
    pub gpio_b14: GpioPinPeriph<GpioB14>,
    /// Blue user button.
    pub gpio_c13: GpioPinPeriph<GpioC13>,
}

impl GpioPins {
    /// Creates a new [`GpioPins`].
    #[inline]
    pub fn new(res: GpioPinsRes) -> Self {
        Self(res)
    }

    /// Releases resources.
    #[inline]
    pub fn free(self) -> GpioPinsRes {
        self.0
    }

    /// Initializes GPIO pins.
    pub fn init(
        &self,
        _gpio_b_en: &inventory::Token<GpioHeadEn<GpioBHead>>,
        _gpio_c_en: &inventory::Token<GpioHeadEn<GpioCHead>>,
    ) {
        self.0.gpio_b7.gpio_moder_moder.modify(|r| {
            self.0.gpio_b7.gpio_moder_moder.write(r, 0b01);
            self.0.gpio_b14.gpio_moder_moder.write(r, 0b01);
        });
        self.0.gpio_b7.gpio_otyper_ot.modify(|r| {
            self.0.gpio_b7.gpio_otyper_ot.clear(r);
            self.0.gpio_b14.gpio_otyper_ot.clear(r);
        });
        self.0.gpio_b7.gpio_ospeedr_ospeedr.modify(|r| {
            self.0.gpio_b7.gpio_ospeedr_ospeedr.write(r, 0b00);
            self.0.gpio_b14.gpio_ospeedr_ospeedr.write(r, 0b00);
        });
        self.0.gpio_b7.gpio_pupdr_pupdr.modify(|r| {
            self.0.gpio_b7.gpio_pupdr_pupdr.write(r, 0b00);
            self.0.gpio_b14.gpio_pupdr_pupdr.write(r, 0b00);
        });
        // -------------
        self.0.gpio_c7.gpio_moder_moder.modify(|r| {
            self.0.gpio_c7.gpio_moder_moder.write(r, 0b01); // Output
        });
        self.0.gpio_c7.gpio_otyper_ot.modify(|r| {
            self.0.gpio_c7.gpio_otyper_ot.clear(r);
        });
        self.0.gpio_c7.gpio_ospeedr_ospeedr.modify(|r| {
            self.0.gpio_c7.gpio_ospeedr_ospeedr.write(r, 0b00);
        });
        self.0.gpio_c7.gpio_pupdr_pupdr.modify(|r| {
            self.0.gpio_c7.gpio_pupdr_pupdr.write(r, 0b00);
        });
        // -------------
        self.0.gpio_c13.gpio_moder_moder.modify(|r| {
            self.0.gpio_c13.gpio_moder_moder.write(r, 0b00); // Input
        });
        self.0.gpio_c13.gpio_pupdr_pupdr.modify(|r| {
            self.0.gpio_c13.gpio_pupdr_pupdr.write(r, 0b10);
        });
    }

    /// Sets the output `value` for the `pin`.
    pub fn output(
        &self,
        //_gpio_b_en: &inventory::Token<GpioHeadEn<GpioBHead>>,
        //_gpio_c_en: &inventory::Token<GpioHeadEn<GpioCHead>>,
        pin: u8,
        value: bool,
    ) {
        match pin {
            1 => {
                if value {
                    self.0.gpio_c7.gpio_bsrr_bs.set_bit();
                } else {
                    self.0.gpio_c7.gpio_bsrr_br.set_bit();
                }
            }
            2 => {
                if value {
                    self.0.gpio_b7.gpio_bsrr_bs.set_bit();
                } else {
                    self.0.gpio_b7.gpio_bsrr_br.set_bit();
                }
            }
            3 => {
                if value {
                    self.0.gpio_b14.gpio_bsrr_bs.set_bit();
                } else {
                    self.0.gpio_b14.gpio_bsrr_br.set_bit();
                }
            }
            _ => panic!("invalid gpio pin"),
        }
    }
}
