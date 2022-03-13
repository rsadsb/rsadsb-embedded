#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

use cortex_m_rt::entry;
use core::fmt::Write;
use heapless::Vec;
use rtt_target::{rtt_init_print, rprintln};
use panic_rtt_target as _;
use adsb_deku::Frame;
use adsb_deku::deku::DekuContainerRead;
use alloc_cortex_m::CortexMHeap;
use cortex_m::asm;
use core::alloc::Layout;

use microbit::{
    hal::prelude::*,
    hal::uarte,
    hal::uarte::{Baudrate, Parity},
};

mod serial_setup;
use serial_setup::UartePort;

// this is the allocator the application will use
#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

// define what happens in an Out Of Memory (OOM) condition
#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
    asm::bkpt();

    loop {}
}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();

    let mut serial = {
        let serial = uarte::Uarte::new(
            board.UARTE0,
            board.uart.into(),
            Parity::EXCLUDED,
            Baudrate::BAUD115200,
        );
        UartePort::new(serial)
    };

    // A buffer with 32 bytes of capacity
    let mut buffer: Vec<u8, 14> = Vec::new();

    loop {
        // We assume that the receiving cannot fail
        let byte = nb::block!(serial.read()).unwrap();

        if buffer.push(byte).is_err() {
            write!(serial, "error: buffer full\r\n").unwrap();
        }

        rprintln!("{:x?}", buffer);
        if buffer.len() == 14 {
            rprintln!("before");
            if let Ok(frame) = Frame::from_bytes((&buffer, 0)) {
                rprintln!("{}", frame.1);
            }
            buffer.clear();
        }
    }
}
