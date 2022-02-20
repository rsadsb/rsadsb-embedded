//! Uses the StatefulOutputPin embedded_hal trait to toggle the pin
//! On the stm32 discovery board this is the "south" led
//! Target board: STM32F3DISCOVERY
#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;
use self::alloc::vec;

// core
use core::alloc::Layout;

// third party
use adsb_deku::deku::DekuContainerRead;
use adsb_deku::Frame;
use alloc_cortex_m::CortexMHeap;
use cortex_m::asm;
use cortex_m_rt::entry;
use hal::pac;
use hal::prelude::*;
use hexlit::hex;

use stm32f3xx_hal as hal;

use panic_semihosting as _;

// this is the allocator the application will use
#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

const HEAP_SIZE: usize = 1024; // in bytes

// define what happens in an Out Of Memory (OOM) condition
#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
    asm::bkpt();

    loop {}
}

#[entry]
fn main() -> ! {
    unsafe { ALLOCATOR.init(cortex_m_rt::heap_start() as usize, HEAP_SIZE) }
    let dp = pac::Peripherals::take().unwrap();

    let v = vec![1_000_000, 2_000_000, 8_000_000];

    let bytes = hex!("8D40621D58C382D690C8AC2863A7");
    let frame = Frame::from_bytes((&bytes, 0));

    let mut rcc = dp.RCC.constrain();
    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);

    let mut led = gpioe
        .pe13
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);

    led.set_low().unwrap();

    loop {
        for t in &v {
            led.toggle().unwrap();
            cortex_m::asm::delay(*t);
            // Toggle by hand.
            // Uses `StatefulOutputPin` instead of `ToggleableOutputPin`.
            // Logically it is the same.
            if led.is_set_low().unwrap() {
                led.set_high().unwrap();
            } else {
                led.set_low().unwrap();
            }
            cortex_m::asm::delay(*t);
        }
    }
}
