# Experimental rsadsb embedded testing

Experimental applications using the `no_std` feature of [rsadsb/adsb_deku](https://github.com/rsadsb/adsb_deku).

##  rsadsb-serial-comm
Much like a `dump1090`-like program, dump the ADS-B bytes from [dump1090_rs](https://github.com/rsadsb/dump1090_rs) into a serial connection.

### Usage
```
cd rsadsb-serial-comm
cargo r
```

## bbc microbit-v2
Display the current position of the closest plane on the led array.

![microbitv2 example](media/microbitv2_03_20_22.gif)

### Usage
You must install [cargo-embed](https://github.com/probe-rs/cargo-embed) for the flashing of the board with our firmware.

```
cd microbitv2
cargo embed --release
```

## stm32f3discovery
Using serial line on the board, show ADS-B packets on the rtt(real-time-terminal) display, flashing the LED when received.

## Connections
| USB Serial Adapter | stm32 Pin  |
| ------------------ | ---------- |
| Tx                 | PA9        |
| Rx                 | PA10       |
| GND                | GND        |


### Usage
You must install [cargo-embed](https://github.com/probe-rs/probe-rs/tree/master/cargo-embed) for the flashing of the board with our firmware.

```
cd stm32f3discovery
cargo embed --release
```
