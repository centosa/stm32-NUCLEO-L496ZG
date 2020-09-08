//! System associated helper functions.

use crate::consts::{HSE_CLK, HSI16_CLK};
use crate::tasks::root::SystemRes;
use crate::thr;
use drone_core::log;
use drone_cortexm::swo;
use drone_cortexm::{fib, reg::prelude::*, thr::prelude::*};
use drone_stm32_map::periph::sys_tick::SysTickPeriph;
use futures::prelude::*;

/// An error returned when a receiver has missed too many ticks.
#[derive(Debug)]
pub struct TickOverflow;

/// System.
pub struct System {}

impl System {
    /// Creates a new [`System`].
    #[inline]
    pub fn new() -> Self {
        Self {}
    }

    /// Apply the current clock tree configuration.
    pub fn apply_clock_config(res: &SystemRes) {
        res.flash.set_latency(5);
        res.rcc.init(res);
        res.lse.init(res);
        res.msi.init(res);
        // Start HSI16 only if used as clock source.
        if res.pllsrc == 0b10 || res.clksrc == 0b01 {
            res.hsi16.init(res);
            swo::flush();
            swo::update_prescaler(16_000_000 / log::baud_rate!() - 1);
        }
        // Start pll only if used as clock source.
        if res.clksrc == 0b11 {
            res.pll.disable();
            res.pll.init(res);
            res.pll.enable();
        }
        swo::flush();
        swo::update_prescaler(Self::calculate_hclk(res) / log::baud_rate!() - 1);
        res.flash.set_latency(Self::calculate_latency(res));
    }

    /// Resets the RCC.
    pub fn reset_rcc(res: &SystemRes) {
        res.rcc.reset();
        res.lse.reset();
        res.pll.reset();
        res.msi.reset();
        res.hsi16.reset();
        swo::flush();
        swo::update_prescaler(4_000_000 / log::baud_rate!() - 1);
    }

    /// Set flash read access latency.
    // To correctly read data from Flash memory, the number of
    // wait states (LATENCY) must be correctly programmed
    pub fn calculate_latency(res: &SystemRes) -> u32 {
        let msi_range_table: [i32; 12] = [
            100_000, 200_000, 400_000, 800_000, 1_000_000, 2_000_000, 4_000_000, 8_000_000,
            16_000_000, 24_000_000, 32_000_000, 48_000_000,
        ];
        let mut _hclk: u32 = 4_000_000;
        let mut _pllvco = 0;
        let msi_clk: u32 = msi_range_table[res.msi.read_msirange() as usize] as u32;
        match res.clksrc {
            0b00 => {
                // MSI oscillator used as system clock.
                _hclk = msi_clk;
            }
            0b01 => {
                // HSI16 oscillator used as system clock.
                _hclk = HSI16_CLK;
            }
            0b10 => {
                // HSE used as system clock.
                _hclk = HSE_CLK;
            }
            0b11 => {
                // PLL used as system clock.
                println!("PLL used as system clock");
                match res.pllsrc {
                    0b01 => {
                        // MSI clock selected as PLL, PLLSAI1 and PLLSAI2 clock entry.
                        _pllvco = msi_clk / res.pllm;
                    }
                    0b10 => {
                        println!("HSI16 used as PLL entry");
                        // HSI16 clock selected as PLL, PLLSAI1 and PLLSAI2 clock entry
                        _pllvco = HSI16_CLK / res.pllm;
                    }
                    0b11 => {
                        _pllvco = HSE_CLK / res.pllm;
                    }
                    _ => {
                        // No clock sent to PLL, PLLSAI1 and PLLSAI2
                        _pllvco = 0;
                    }
                }
                // Multiply by value of main PLL multiplication factor.
                _pllvco = _pllvco * res.plln;

                // Divide by main PLL division factor.
                _hclk = _pllvco / res.pllr;
                //unsafe { llvm_asm!("bkpt" :::: "volatile") };
            }
            _ => _hclk = msi_clk,
        }

        // Return the correct number of wait states according to ref manual.
        println!("hclk for latency {}", _hclk);
        _hclk / 16_000_000
    }

    pub fn calculate_hclk(res: &SystemRes) -> u32 {
        let msi_range_table: [i32; 12] = [
            100_000, 200_000, 400_000, 800_000, 1_000_000, 2_000_000, 4_000_000, 8_000_000,
            16_000_000, 24_000_000, 32_000_000, 48_000_000,
        ];
        let mut _hclk: u32 = 4_000_000;
        let mut _pllvco = 0;
        let msi_clk = msi_range_table[res.msi.read_msirange() as usize] as u32;
        // Check which clock source is used as system clock.
        match res.rcc.read_sws() {
            0b00 => {
                // MSI oscillator used as system clock.
                _hclk = msi_clk;
            }
            0b01 => {
                // HSI16 oscillator used as system clock.
                _hclk = HSI16_CLK;
            }
            0b10 => {
                // HSE used as system clock.
                _hclk = HSE_CLK;
            }
            0b11 => {
                // PLL used as system clock.
                match res.pll.read_pllsrc() {
                    0b01 => {
                        // MSI clock selected as PLL, PLLSAI1 and PLLSAI2 clock entry.
                        _pllvco = msi_clk / res.pllm;
                    }
                    0b10 => {
                        // HSI16 clock selected as PLL, PLLSAI1 and PLLSAI2 clock entry
                        _pllvco = HSI16_CLK / res.pllm;
                    }
                    0b11 => {
                        // HSE clock selected as PLL, PLLSAI1 and PLLSAI2 clock entry.
                        // CAUTION:
                        // User has to ensure that HSE_VALUE is same as the real
                        // frequency of the crystal used. Otherwise, this function may
                        // have wrong result.
                        _pllvco = HSE_CLK / res.pllm;
                    }
                    _ => {
                        // No clock sent to PLL, PLLSAI1 and PLLSAI2
                        _pllvco = 0;
                    }
                }
                // Multiply by value of main PLL multiplication factor ( as set in PLLCFGR field.)
                _pllvco = _pllvco * res.pll.read_plln();

                // Divide by main PLL division factor.
                // The value read from register's 2-bit field has to be scaled.
                // 0b00: PLLR = 2, 0b01: PLLR = 4, 0b10: PLLR = 6, 0b11: PLLR = 8
                _hclk = _pllvco / ((res.pll.read_pllr() + 1) * 2);
            }
            _ => _hclk = msi_clk,
        }
        _hclk
    }

    /// Millisecond delay.
    pub async fn delay(
        millis: u32,
        hclk: u32,
        sys_tick: &SysTickPeriph,
        thr_sys_tick: thr::SysTick,
    ) -> () {
        let mut tick_stream = thr_sys_tick
            .add_pulse_try_stream(|| Err(TickOverflow), fib::new_fn(|| fib::Yielded(Some(1))));
        sys_tick.stk_val.store(|r| r.write_current(0));
        sys_tick
            .stk_load
            .store(|r| r.write_reload(millis * (hclk / 8000)));
        sys_tick.stk_ctrl.store(|r| {
            r.set_tickint() // Counting down to 0 triggers the SysTick interrupt
                .set_enable() // Start the counter in a multi-shot way
        });
        tick_stream.next().await;
    }
}
