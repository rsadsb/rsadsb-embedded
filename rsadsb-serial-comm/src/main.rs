use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::time::Duration;

use clap::Parser;
use serialport::{DataBits, Parity, StopBits};

#[derive(Debug, Parser)]
#[clap(
    version,
    name = "1090",
    author = "wcampbell0x2a",
    about = "Dump ADS-B protocol info from demodulator"
)]
struct Options {
    /// ip address of ADS-B demodulated bytes server
    #[clap(long, default_value = "localhost")]
    host: String,
    /// port of ADS-B demodulated bytes server
    #[clap(long, default_value = "30002")]
    port: u16,
    /// Panic on adsb_deku::Frame::fmt::Display not implemented
    #[clap(long)]
    panic_display: bool,
    /// Panic on adsb_deku::Frame::from_bytes() error
    #[clap(long)]
    panic_decode: bool,
    /// Display debug of adsb::Frame
    #[clap(long)]
    debug: bool,
}

fn main() {
    let options = Options::parse();

    let mut port = serialport::new("/dev/ttyACM0", 115200)
        .timeout(Duration::from_millis(10))
        .data_bits(DataBits::Eight)
        .parity(Parity::None)
        .stop_bits(StopBits::One)
        .open()
        .unwrap();

    let stream = TcpStream::connect((options.host, options.port)).unwrap();
    stream
        .set_read_timeout(Some(std::time::Duration::from_millis(50)))
        .unwrap();
    let mut reader = BufReader::new(stream);
    let mut input = String::new();

    loop {
        input.clear();
        if let Ok(len) = reader.read_line(&mut input) {
            if len == 0 {
                continue;
            }
            // convert from string hex -> bytes
            let hex = &mut input.to_string()[1..len - 2].to_string();
            println!("{}", hex.to_lowercase());
            let bytes = if let Ok(bytes) = hex::decode(&hex) {
                bytes
            } else {
                continue;
            };

            // check for all 0's
            if bytes.iter().all(|&b| b == 0) {
                continue;
            }

            port.write_all(&bytes).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(50));

            input.clear();
        }
    }
}
