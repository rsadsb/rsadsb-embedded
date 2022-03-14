#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

use adsb_deku::deku::DekuContainerRead;
use adsb_deku::Frame;
use rsadsb_common::Airplanes;
use alloc_cortex_m::CortexMHeap;
use core::alloc::Layout;
use core::mem::MaybeUninit;
use core::fmt::Write;
use cortex_m::asm;
use cortex_m_rt::entry;
use heapless::Vec;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use microbit::{
    hal::prelude::*,
    hal::uarte,
    hal::uarte::{Baudrate, Parity},
};

mod serial_setup;
use serial_setup::UartePort;

const HEAP_SIZE: usize = 4096*2;

const LAT: f64 = 0;
const LONG: f64 = 0;

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

    static mut HEAP: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    unsafe { ALLOCATOR.init((&mut HEAP).as_ptr() as usize, HEAP_SIZE) }
    let mut adsb_airplanes = Airplanes::new();

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

    let mut buffer: Vec<u8, 14> = Vec::new();

    loop {
        // We assume that the receiving cannot fail
        let byte = nb::block!(serial.read()).unwrap();

        if buffer.push(byte).is_err() {
            write!(serial, "error: buffer full\r\n").unwrap();
        }

        if buffer.len() == 14 {
            rprintln!("bytes: {:x?}", buffer);
            match Frame::from_bytes((&buffer, 0)) {
                Ok(frame) => {
                    rprintln!("{}", frame.1);
                    rprintln!("!: {:?}", adsb_airplanes.0.keys());
                    adsb_airplanes.action(frame.1, (LAT, LONG));
                    for (key, value) in &adsb_airplanes.0 {
                        if let Some(distance) = value.coords.kilo_distance {
                            rprintln!("[{}], {} km", key, distance);
                        }
                    }
                },
                Err(e) => rprintln!("[!] ERROR: {:?}", e),
            }
            buffer.clear();
        }
    }
}
