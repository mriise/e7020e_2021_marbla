#![no_main]
#![no_std]

use cortex_m_semihosting::hprintln;
use panic_halt as _;
use stm32f2;

#[rtic::app(device = stm32f2)]
const APP: () = {
    #[init]
    fn init(_cx: init::Context) {
        for a in 0..11 {
            hprintln!("RTIC Says Hello, to all students!! {}", a).unwrap();
        }
    }
};
