# NUCLEO-L496ZG Sample Application
Drone-OS firmware example for STM32 NUCLEO-L496ZG and NUCLEO-L496ZG-P boards.
This project was also tested on the STM32 NUCLEO-L4R5I-P without modifications.

## Difficulty level
Basic 'blinky' type application in combination with a user button and a 
dynamic clock tree configuration that changes when user button is clicked.

## Summary
- Configure the clock tree to run the mcu at 4, 16, 48, 80 MHz dynamically
  selectable at run-time with a button click.
- Configure the LSE (32MHz External crystal) as the external clock source.
- Configure 3 GPIO output pins to drive the on-board red/green/blue user leds.
- Write log message to SWO output.
- Configure the EXTI interrupt for the gpio that is assigned to the button.
- Listen to the systick and to the button click event stream simultaneously.

This firmware is written with the 'official' Drone-OS crates. No additional
crates were used other than those normally used by Drone-OS.

## Toolchain
The project is currently dependent on nightly-2020-04-30. It will be upgraded
to the latest nightly as soon as the corresponding Drone-OS crates are released.

## License
Licensed under either of

Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)
at your option.

## Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
