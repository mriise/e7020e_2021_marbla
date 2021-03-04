//! rtic_bare6.rs
//!
//! Clocking
//!
//! What it covers:
//! - using svd2rust generated API
//! - using the stm32f4xx-hal to set clocks
//! - routing the clock to a PIN for monitoring by an oscilloscope

#![no_main]
#![no_std]

use panic_rtt_target as _;
use rtic::cyccnt::{Instant, U32Ext as _};
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::{
    prelude::*,
    stm32::{self, GPIOC, RCC},
};

const OFFSET: u32 = 8_000_000;

#[rtic::app(device = stm32f4xx_hal::stm32, monotonic = rtic::cyccnt::CYCCNT, peripherals = true)]
const APP: () = {
    struct Resources {
        // late resources
        GPIOA: stm32::GPIOA,
    }
    #[init(schedule = [toggle])]
    fn init(cx: init::Context) -> init::LateResources {
        rtt_init_print!();
        rprintln!("init");

        let mut core = cx.core;
        let device = cx.device;

        // Initialize (enable) the monotonic timer (CYCCNT)
        core.DCB.enable_trace();
        core.DWT.enable_cycle_counter();

        // semantically, the monotonic timer is frozen at time "zero" during `init`
        // NOTE do *not* call `Instant::now` in this context; it will return a nonsense value
        let now = cx.start; // the start time of the system

        // Schedule `toggle` to run 8e6 cycles (clock cycles) in the future
        cx.schedule.toggle(now + OFFSET.cycles()).unwrap();

        // setup LED
        // power on GPIOA, RM0368 6.3.11
        device.RCC.ahb1enr.modify(|_, w| w.gpioaen().set_bit());
        // configure PA5 as output, RM0368 8.4.1
        device.GPIOA.moder.modify(|_, w| w.moder5().bits(1));

        clock_out(&device.RCC, &device.GPIOC);

        let rcc = device.RCC.constrain();

        let _clocks = rcc.cfgr.freeze();

        // Set up the system clock. 48 MHz?
        // let _clocks = rcc
        //     .cfgr
        //     .sysclk(48.mhz())
        //     .pclk1(24.mhz())
        //     .freeze();

        // pass on late resources
        init::LateResources {
            GPIOA: device.GPIOA,
        }
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        rprintln!("idle");
        loop {
            continue;
        }
    }

    #[task(resources = [GPIOA], schedule = [toggle])]
    fn toggle(cx: toggle::Context) {
        static mut TOGGLE: bool = false;
        rprintln!("toggle  @ {:?}", Instant::now());

        if *TOGGLE {
            cx.resources.GPIOA.bsrr.write(|w| w.bs5().set_bit());
        } else {
            cx.resources.GPIOA.bsrr.write(|w| w.br5().set_bit());
        }

        *TOGGLE = !*TOGGLE;
        cx.schedule.toggle(cx.scheduled + OFFSET.cycles()).unwrap();
    }

    extern "C" {
        fn EXTI0();
    }
};

// see the Reference Manual RM0368 (www.st.com/resource/en/reference_manual/dm00096844.pdf)
// rcc,     chapter 6
// gpio,    chapter 8

fn clock_out(rcc: &RCC, gpioc: &GPIOC) {
    // output MCO2 to pin PC9

    // mco2 	: SYSCLK = 0b00
    // mcopre 	: divide by 4 = 0b110
    rcc.cfgr
        .modify(|_, w| unsafe { w.mco2().bits(0b00).mco2pre().bits(0b110) });

    // power on GPIOC, RM0368 6.3.11
    rcc.ahb1enr.modify(|_, w| w.gpiocen().set_bit());

    // MCO_2 alternate function AF0, STM32F401xD STM32F401xE data sheet
    // table 9
    // AF0, gpioc reset value = AF0

    // configure PC9 as alternate function 0b10, RM0368 6.2.10
    gpioc.moder.modify(|_, w| w.moder9().bits(0b10));

    // otyper reset state push/pull, in reset state (don't need to change)

    // ospeedr 0b11 = very high speed
    gpioc.ospeedr.modify(|_, w| w.ospeedr9().bits(0b11));
}

// 1. In this example you will use RTT.
//
//    > cargo run --example rtic_bare6
//
//    Confirm that your RTT traces the init, idle and led on/off.
//
//    What is the (default) MCU (SYSCLK) frequency?
//
//    ** your answer here **
//
//    What is the (default) DWT CYCCNT frequency?
//
//    ** your answer here **
//
//    What is the frequency of blinking?
//
//    ** your answer here **
//
//    commit your answers (bare6_1)
//
// 2. Now connect an oscilloscope to PC9, which is set to
//    output the MCO2.
//
//    Compute the value of SYSCLK based on the oscilloscope reading
//
//    ** your answer here **
//
//    What is the peak to peak (voltage) reading of the signal?
//
//    ** your answer here **
//
//    Make a folder called "pictures" in your git project.
//    Make a screen dump or photo of the oscilloscope output.
//    Save the the picture as "bare_6_16mhz_high_speed".
//
//    Commit your answers (bare6_2)
//
// 3. Now run the example in 48Mz, by commenting out line 56, and un-commenting
//    lines 58-63.
//`
//    What is the frequency of blinking?
//
//    ** your answer here **
//
//    Commit your answers (bare6_3)
//
//    Now change the constant `OFFSET` so you get the same blinking frequency as in 1.
//    Test and validate that you got the desired behavior.
//
//    Commit your answers (bare6_3)
//
// 4. Repeat experiment 2
//
//    What is the frequency of MCO2 read by the oscilloscope?
//
//    ** your answer here **
//
//    Compute the value of SYSCLK based on the oscilloscope reading.
//
//    ** your answer here **
//
//    What is the peak to peak reading of the signal?
//
//    ** your answer here **
//
//    Make a screen dump or photo of the oscilloscope output.
//    Save the the picture as "bare_6_64mhz_high_speed".
//
//    Commit your answers (bare6_4)
//
// 5. In the `clock_out` function, the setup of registers is done through
//    setting bit-pattens manually, e.g.
//     rcc.cfgr
//        .modify(|_, w| unsafe { w.mco2().bits(0b00).mco2pre().bits(0b110) });
//
//    However based on the vendor SVD file the svd2rust API provides
//    a better abstraction, based on pattern enums and functions.
//
//    To view the API you can generate documentation for your crate:
//
//    > cargo doc --open
//
//    By searching for `mco2` you find the enumerations and functions.
//    So here
//       `w.mco2().bits{0b00}` is equivalent to
//       `w.mco2().sysclk()` and improves readability.
//
//    Replace all bit-patterns used by the function name equivalents.
//
//    Test that the application still runs as before.
//
//    Commit your code (bare6_5)
//
// 6. Discussion
//
//    In this exercise, you have learned to use the stm32f4xx-hal
//    to set the clock speed of your MCU.
//
//    You have also learned how you can monitor/validate MCU clock(s) on pin(s)
//    connected to an oscilloscope.
//
//    You have also learned how you can improve readability of your code
//    by leveraging the abstractions provided by the PAC.
//
//    As mentioned before the PACs are machine generated by `svd2rust`
//    from vendor provided System View Desciptions (SVDs).
//
//    The PACs provide low level peripheral access abstractions, while
//    the HALs provide higher level abstractions and functionality.