//! Project constants.

/// SWO baud rate.
pub const SWO_BAUD_RATE: usize = 115_200;

/// MSI (multispeed internal) RC oscillator clock frequency.
pub const MSI_CLK: u32 = 8_000_000;

// HSI16 internal 16 MHz RC Oscillator.
pub const HSI16_CLK: u32 = 16_000_000;

// HSI48 clock (only valid for STM32L49x/L4Ax devices)
pub const HSI48_CLK: u32 = 48_000_000;

// HSE high speed external clock (not present on Nucleo-144)
pub const HSE_CLK: u32 = 48_000_000;
