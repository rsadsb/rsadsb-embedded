#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

use panic_semihosting as _;
use stm32f3xx_hal as hal;

// core
use core::alloc::Layout;

// third party
use adsb_deku::deku::DekuContainerRead;
use adsb_deku::Frame;
use alloc_cortex_m::CortexMHeap;
use cortex_m::{asm, singleton};
use cortex_m_rt::entry;
use hal::prelude::*;
use hal::{pac, serial::Serial};

use rtt_target::{rprintln, rtt_init_print};

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
    rtt_init_print!();

    let dp = pac::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.constrain();
    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);

    let mut led = gpioe
        .pe13
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);

    led.set_low().unwrap();

    let mut flash = dp.FLASH.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);

    let pins = (
        gpioa
            .pa9
            .into_af_push_pull(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrh),
        gpioa
            .pa10
            .into_af_push_pull(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrh),
    );

    let serial = Serial::new(dp.USART1, pins, 9600.Bd(), clocks, &mut rcc.apb2);

    let (_tx, rx) = serial.split();

    let dma1 = dp.DMA1.split(&mut rcc.ahb);

    // DMA channel selection depends on the peripheral:
    let (_tx_channel, rx_channel) = (dma1.ch4, dma1.ch5);

    //let tx_buf = singleton!(: [u8; 14] = [0; 14]).unwrap();
    let rx_buf = singleton!(: [u8; 14] = [0; 14]).unwrap();

    //let rx_buf = alloc::vec![0; 14];
    //let mut send = (tx_buf, tx_channel, tx);
    let mut recv = (rx_buf, rx_channel, rx);

    loop {
        //let (tx_buf, tx_channel, tx) = send;
        let (rx_buf, rx_channel, rx) = recv;

        //let sending = tx.write_all(tx_buf, tx_channel);
        let receiving = rx.read_exact(rx_buf, rx_channel);
        //send = sending.wait();
        recv = receiving.wait();
        rprintln!("message: {:x?}", recv.0);

        led.toggle().unwrap();
        rprintln!("decoding");
        if let Ok(frame) = Frame::from_bytes((recv.0, 0)) {
            rprintln!("{}", frame.1);
        }
        if led.is_set_low().unwrap() {
            led.set_high().unwrap();
        } else {
            led.set_low().unwrap();
        }
    }
}
