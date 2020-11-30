//! The root task.

use crate::{
    drv::{
        exti::{ExtiDrv, ExtiSetup},
        flash::Flash,
        gpio::GpioHead,
        hsi16::Hsi16,
        lse::Lse,
        msi::Msi,
        pll::Pll,
        rcc::Rcc,
    },
    drv_gpio_pins,
    sys::{gpio_pins::GpioPins, system::System},
    thr,
    thr::{Thrs, ThrsInit},
    Regs,
};

use drone_cortexm::processor::fpu_init;
use drone_cortexm::{fib, reg::prelude::*, thr::prelude::*};
use drone_stm32_map::periph::exti::periph_exti13;
use drone_stm32_map::periph::exti::Exti13;
use drone_stm32_map::periph::gpio::{periph_gpio_b_head, periph_gpio_c_head};
use drone_stm32_map::periph::sys_tick::{periph_sys_tick, SysTickPeriph};

use futures::prelude::*;
use futures::select_biased;

enum Event {
    Tick,
    Push,
}

enum ClockMode {
    Reset4MHz,
    Slow16MHz,
    Medium48MHz,
    Full80MHz,
}

/// An error returned when a receiver has missed too many ticks.
#[derive(Debug)]
pub struct TickOverflow;

/// System Resources
pub struct SystemRes {
    pub pll: Pll,
    pub hsi16: Hsi16,
    pub msi: Msi,
    pub lse: Lse,
    pub rcc: Rcc,
    pub flash: Flash,
    pub pllm: u32,
    pub plln: u32,
    pub pllp: u32,
    pub pllq: u32,
    pub pllr: u32,
    pub msirange: u32,
    pub clksrc: u32,
    pub pllsrc: u32,
}

#[allow(unused_labels)]
#[inline(never)]
pub fn handler(reg: Regs, thr_init: ThrsInit) {
    let mut clock_mode = ClockMode::Reset4MHz;

    // Allocate the clock control resources.
    let mut res = SystemRes {
        // ----------------------
        // -- Clocks.
        // The internal PLLs can be used to multiply the HSI16, HSE or MSI
        // output clock frequency.
        pll: Pll::new(periph_pll!(reg)),
        // The HSI16 clock signal is generated from an internal 16 MHz RC Oscillator.
        hsi16: Hsi16::new(periph_hsi16!(reg)),
        // The MSI clock signal is generated from an internal RC oscillator.
        // Its frequency range can be adjusted by software.
        msi: Msi::new(periph_msi!(reg)),
        // The LSE crystal is a 32.768 kHz Low Speed External crystal or ceramic resonator.
        // It is available on the Nucleo board.
        lse: Lse::new(periph_lse!(reg)),
        // The RCC component.
        rcc: Rcc::new(periph_rcc!(reg)),
        // The flash component,
        flash: Flash::new(periph_flash!(reg)),
        // ----------------------
        // -- Factors and selectors.
        // CAUTION: Setting wrong values may make your system unusable.
        // Read the reference manual for detailed information.
        pllm: 1,          // (PLLM) Division factor for the main PLL and audio PLL
        plln: 10,         // (PLLN) Main PLL multiplication factor for VCO.
        pllp: 2,          // (PLLP) Main PLL division factor.
        pllq: 2,          // (PLLQ) Main PLL division factor for PLL48M1CLK.
        pllr: 2,          // (PLLR) Main PLL division factor for PLLCLK (system clock).
        msirange: 0b0110, // 12 possible ranges from 0 to 48 MHz for the MSI
        // Possible values for clksrc:
        // 00: MSI oscillator used as system clock.
        // 01: HSI16 oscillator used as system clock.
        // 10: HSE used as system clock.
        // 11: PLL used as system clock.
        clksrc: 0b00, // Field RCC_CFGR_SW in ref. manual.
        // Possible values for pllsrc:
        // 00: No clock sent to PLL, PLLSAI1 and PLLSAI2.
        // 01: MSI clock selected as PLL, PLLSAI1 and PLLSAI2 clock entry.
        // 10: HSI16 clock selected as PLL, PLLSAI1 and PLLSAI2 clock entry.
        // 11: HSE clock selected as PLL, PLLSAI1 and PLLSAI2 clock entry.
        pllsrc: 0b00, // Field RCC_PLLCFGR_PLLSRC in ref. manual.
    };

    // The on-board user LEDs are connected to GPIO banks B and C.
    // Create register and pins mapping component.
    let gpio_pins = drv_gpio_pins!(reg);
    let mut gpio_b = GpioHead::new(periph_gpio_b_head!(reg));
    let mut gpio_c = GpioHead::new(periph_gpio_c_head!(reg));
    // Enable and initialize.
    let gpio_b_en = gpio_b.enable();
    let gpio_c_en = gpio_c.enable();
    gpio_pins.init(gpio_b_en.inventory_token(), gpio_c_en.inventory_token());

    let sys_tick = periph_sys_tick!(reg);
    let (thr, scb) = thr::init_extended(thr_init);
    thr.hard_fault.add_once(|| panic!("Hard Fault"));

    scb.scb_ccr_div_0_trp.set_bit();
    unsafe {
        fpu_init(true);
    }

    // Enable the system configuration controller clock.
    reg.rcc_apb2enr.syscfgen.set_bit();

    // Setup fault handlers.
    thr.hard_fault.add_once(|| panic!("Hard Fault"));

    // Exti configuration for the user button.
    let exti13 = ExtiDrv::init(ExtiSetup {
        exti: periph_exti13!(reg),
        exti_int: thr.exti_15_10,
        config: 0b0010, // PC13 pin.
        falling: false, // trigger the interrupt on a falling edge.
        rising: true,   // don't trigger the interrupt on a rising edge.
    });

    'user_button_pressed: loop {
        // Reset the clock control registers to their default.
        System::reset_rcc(&res);
        System::delay(20, 4_000_000, &sys_tick, thr.sys_tick).root_wait();

        // Apply the current clock tree configuration.
        System::apply_clock_config(&res);

        // Calculate the configured clock speed.
        let hclk = System::calculate_hclk(&res);

        System::delay(20, hclk, &sys_tick, thr.sys_tick).root_wait();

        // Adapt SWO clock configuration to current speed.
        println!("speed {}", hclk);

        listen(&sys_tick, &thr, thr.sys_tick, &exti13, &gpio_pins, hclk).root_wait();

        // Set different configuration for the clock tree
        match clock_mode {
            ClockMode::Reset4MHz => {
                clock_mode = ClockMode::Slow16MHz; // <- new mode.
                res.pllsrc = 0b00; // PLL without input.
                res.clksrc = 0b01; // Use HSI16 clock source.
                res.msirange = 0b0110; // MSI reset value
                gpio_pins.output(1, true);
                gpio_pins.output(2, false);
            }
            ClockMode::Slow16MHz => {
                clock_mode = ClockMode::Medium48MHz; // <- new mode.
                res.pllsrc = 0b00; // PLL without input.
                res.clksrc = 0b00; // Use MSI clock source.
                res.msirange = 0b1011; // MSI 48MHz mode.
                gpio_pins.output(1, false);
                gpio_pins.output(2, true);
            }
            ClockMode::Medium48MHz => {
                clock_mode = ClockMode::Full80MHz; // <- new mode.
                res.pllsrc = 0b10; // HSI16 is PLL clock input.
                res.clksrc = 0b11; // Use PLL output 80 MHz
                res.msirange = 0b0110; // MSI reset value
                gpio_pins.output(1, true);
                gpio_pins.output(2, true);
            }
            ClockMode::Full80MHz => {
                clock_mode = ClockMode::Reset4MHz; // <- new mode.
                res.pllsrc = 0b00; // PLL without input.
                res.clksrc = 0b00; // Use MSI.
                res.msirange = 0b0110; // MSI reset value 4MHz.
                gpio_pins.output(1, false);
                gpio_pins.output(2, false);
            }
        }
    }
}

async fn listen(
    sys_tick: &SysTickPeriph,
    thr: &Thrs,
    thr_sys_tick: thr::SysTick,
    exti13: &ExtiDrv<Exti13, thr::Exti1510>,
    gpio_pins: &GpioPins,
    hclk: u32,
) -> Event {
    println!("enter listen");
    // Attach a listener that will notify us on user button pressed.
    let mut button_stream = exti13.create_saturating_stream();

    // Attach a listener that will notify us on each SYS_TICK interrupt trigger.
    let mut tick_stream = thr_sys_tick.add_pulse_try_stream(
        // This closure will be called when a receiver no longer can store the
        // number of ticks since the last stream poll. If this happens, a
        // `TickOverflow` error will be sent over the stream as is final value.
        || Err(TickOverflow),
        // A fiber that will be called on each interrupt trigger. It sends a
        // single tick over the stream.
        fib::new_fn(|| fib::Yielded(Some(1))),
    );

    // Clear the current value of the timer.
    sys_tick.stk_val.store(|r| r.write_current(0));
    //
    // The duration of setting the led ON is inversely proportional to the
    // MCU clock speed. It shall be:
    //   4.00 seconds when cpu clocks @ 4MHz
    //   1.00 seconds when cpu clocks @ 16MHz
    //   0.33 seconds when cpu clocks @ 48MHz
    //   0.20 seconds when cpu clocks @ 80MHz

    // The trigger is set so that it returns twice per interval
    // at the highest speed, and proportionally more often per interval
    // at lower speeds.
    // That way, the Exti interrupt will happen every 100ms at all speeds
    // and it can be used to for debounceing and doubleclick control.
    let mut trigger = 4_000_000 / 8; // So many systick/sec at 4MHz.
    trigger = trigger / 10; // So many in 100ms at 4MHz.
    trigger = trigger * (hclk / 4_000_000); // More at higher speed

    sys_tick.stk_load.store(|r| r.write_reload(trigger));
    sys_tick.stk_ctrl.store(|r| {
        r.set_tickint() // Counting down to 0 triggers the SysTick interrupt
            .set_enable() // Start the counter in a multi-shot way
    });

    let mut red_led_on = true;
    gpio_pins.output(3, true); // Start with red led ON.

    // Enable the interrupt for the user button.
    thr.exti_15_10.enable_int();

    // Counters
    let mut debounce_protection: i16 = 0;
    let mut doubleclick_protection: i16 = 0;
    let mut ticks_cnt: u32 = 0;

    // Monitored interval lengths (accumulated ticks).
    let debounce_ival = 2;
    let doubleclick_ival = 4;

    // This is dependent on mcu speed:
    let ticks_ival: u32 = 40 / (hclk / 4_000_000);

    'blinky: loop {
        let evt = select_biased! {
            _p = button_stream.next().fuse() => Event::Push,
            _t = tick_stream.next().fuse() => Event::Tick,
        };
        match evt {
            Event::Tick => {
                if debounce_protection > i16::MIN {
                    debounce_protection = debounce_protection - 1;
                };
                if doubleclick_protection < i16::MAX {
                    doubleclick_protection = doubleclick_protection + 1;
                };
                if debounce_protection == 0 && doubleclick_protection >= doubleclick_ival {
                    break 'blinky;
                }
                // The low and the high interval is 'ticks_ival' ticks.
                ticks_cnt = ticks_cnt + 1;
                if ticks_cnt >= ticks_ival {
                    ticks_cnt = 0;
                    match red_led_on {
                        true => {
                            red_led_on = false;
                            gpio_pins.output(3, false);
                        }
                        _ => {
                            red_led_on = true;
                            gpio_pins.output(3, true);
                        }
                    }
                }
            }
            Event::Push => {
                // After disabling the interrupt or after re-enabling 
                // the interrupt, the stream needs to be flushed to protect 
                // the logic during the switching period against mechanical 
                // contact bouncing and doubleclicks.
                if doubleclick_protection > doubleclick_ival {
                    println!("--");
                    thr.exti_15_10.disable_int();
                    debounce_protection = debounce_ival;
                } else {
                    doubleclick_protection = 0;
                    println!("++");
                }
            }
        }
    }
    Event::Push
}
