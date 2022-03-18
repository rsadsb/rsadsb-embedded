#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

use adsb_deku::deku::DekuContainerRead;
use adsb_deku::Frame;
use alloc_cortex_m::CortexMHeap;
use core::alloc::Layout;
use core::fmt::Write;
use core::mem::MaybeUninit;
use cortex_m::asm;
use cortex_m_rt::entry;
use heapless::Vec;
use microbit::{
    board::Board,
    display::blocking::Display,
    hal::prelude::*,
    hal::uarte,
    hal::uarte::{Baudrate, Parity},
    hal::Timer,
};
use panic_rtt_target as _;
use rsadsb_common::Airplanes;
use rtt_target::{rprintln, rtt_init_print};

mod serial_setup;
use serial_setup::UartePort;

const HEAP_SIZE: usize = 1024 * 100;

const LAT: f64 = 0.0;
const LONG: f64 = 0.0;

// this is the allocator the application will use
#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

// define what happens in an Out Of Memory (OOM) condition
#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
    rprintln!("[!] alloc error");
    asm::bkpt();

    loop {}
}

mod leds {
    pub const TOP_RIGHT: [[u8; 5]; 5] = [
        [0, 0, 1, 1, 1],
        [0, 0, 0, 1, 1],
        [0, 0, 1, 0, 1],
        [0, 1, 0, 0, 0],
        [1, 0, 0, 0, 0],
    ];
    pub const TOP_LEFT: [[u8; 5]; 5] = [
        [1, 1, 1, 0, 0],
        [1, 1, 0, 0, 0],
        [1, 0, 1, 0, 0],
        [0, 0, 0, 1, 0],
        [0, 0, 0, 0, 1],
    ];
    pub const BOT_LEFT: [[u8; 5]; 5] = [
        [0, 0, 0, 0, 1],
        [0, 0, 0, 1, 0],
        [1, 0, 1, 0, 0],
        [1, 1, 0, 0, 0],
        [1, 1, 1, 0, 0],
    ];
    pub const BOT_RIGHT: [[u8; 5]; 5] = [
        [1, 0, 0, 0, 0],
        [0, 1, 0, 0, 0],
        [0, 0, 1, 0, 1],
        [0, 0, 0, 1, 1],
        [0, 0, 1, 1, 1],
    ];
}

#[entry]
fn main() -> ! {
    rtt_init_print!();

    static mut HEAP: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    unsafe { ALLOCATOR.init((&mut HEAP).as_ptr() as usize, HEAP_SIZE) }
    let mut adsb_airplanes = Airplanes::new();

    let board = Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);
    let mut serial = {
        let serial = uarte::Uarte::new(
            board.UARTE0,
            board.uart.into(),
            Parity::EXCLUDED,
            Baudrate::BAUD115200,
        );
        UartePort::new(serial)
    };

    let mut display = Display::new(board.display_pins);

    let mut buffer: Vec<u8, 14> = Vec::new();
    loop {
        // We assume that the receiving cannot fail
        let byte = nb::block!(serial.read()).unwrap();

        if buffer.push(byte).is_err() {
            write!(serial, "error: buffer full\r\n").unwrap();
        }

        if buffer.len() == 14 {
            match Frame::from_bytes((&buffer, 0)) {
                Ok(frame) => {
                    adsb_airplanes.action(frame.1, (LAT, LONG));
                    let mut save_position = (None, 400.0);
                    for (key, value) in adsb_airplanes.iter() {
                        // save smallest heading key
                        if let Some(kilo_distance) = value.coords.kilo_distance {
                            if kilo_distance < save_position.1 {
                                save_position = (Some(key), kilo_distance);
                            }
                        }
                    }
                    if save_position.0.is_some() {
                        let position = adsb_airplanes
                            .get(*save_position.0.unwrap())
                            .unwrap()
                            .coords
                            .position;
                        if let Some(position) = position {
                            rprintln!(
                                "[{}] {:.3} {:.3}",
                                save_position.0.unwrap(),
                                position.latitude,
                                position.longitude
                            );
                            let (lat, long) = (position.latitude, position.longitude);

                            let is_top = lat > LAT;
                            let is_right = long > LONG;

                            let leds = if is_top && is_right {
                                leds::TOP_RIGHT
                            } else if is_top && !is_right {
                                leds::TOP_LEFT
                            } else if !is_top && is_right {
                                leds::BOT_RIGHT
                            } else {
                                leds::BOT_LEFT
                            };
                            display.show(&mut timer, leds, 10);
                        }
                    }
                }
                Err(e) => rprintln!("[!] ERROR: {:?}", e),
            }
            buffer.clear();
        }
    }
}
