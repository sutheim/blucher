#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::{ops::Deref, u8};

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{
    gpio,
    spi::{self, Spi},
};
use embassy_time::{Duration, Timer};
use embedded_nrf24l01::{Configuration, CrcMode, DataRate, Payload, NRF24L01};
use gpio::{Level, Output};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialise Peripherals
    let p = embassy_rp::init(Default::default());

    // Create LED
    let mut led = Output::new(p.PIN_25, Level::Low);
    let csn = Output::new(p.PIN_14, Level::Low);
    let ce = Output::new(p.PIN_17, Level::Low);

    let miso = p.PIN_4;
    let mosi = p.PIN_7;
    let clk = p.PIN_6;

    let mut config = spi::Config::default();
    config.frequency = 4_000_000;

    let spi = Spi::new_blocking(p.SPI0, clk, mosi, miso, config);

    let address_main: [u8; 5] = [0x00, 0x00, 0x00, 0x00, 0x00];
    let address_secondary: [u8; 5] = [0x11, 0x11, 0x11, 0x11, 0x11];
    let pipes = (&address_main, &address_secondary);

    let mut nrf24 = NRF24L01::new(ce, csn, spi).unwrap();
    nrf24.set_frequency(108).unwrap();
    nrf24.set_auto_retransmit(0, 0).unwrap();
    nrf24.set_rf(&DataRate::R250Kbps, 3).unwrap();

    nrf24.set_pipes_rx_lengths(&[Some(8); 6]).unwrap();
    nrf24
        .set_pipes_rx_enable(&[false, true, false, false, false, false])
        .unwrap();
    nrf24.set_auto_ack(&[true; 6]).unwrap();
    nrf24.set_crc(CrcMode::TwoBytes).unwrap();

    nrf24.set_tx_addr(pipes.1).unwrap();
    nrf24.set_rx_addr(1, pipes.0).unwrap();

    nrf24.flush_tx().unwrap();
    nrf24.flush_rx().unwrap();

    let mut rx = nrf24.rx().unwrap();

    let mut val_received : &[u8];
    // Loop
    loop {
        Timer::after(Duration::from_millis(5)).await;

        if let Some(_pipe) = rx.can_read().unwrap() {

            if let Ok(payload) = rx.read() {
                if !payload.is_empty() {
                    led.set_high();
                    // let out : Message = postcard::from_bytes(payload.deref()).unwrap();
                    val_received = payload.deref();
                    // let mut data = [0u8; 4096];
                    // data.push_str(core::str::from_utf8(dereffed).unwrap()).unwrap();
                    let string = core::str::from_utf8(val_received).unwrap();
                    info!("Received {}", string);

                    let mut tx = rx.standby().tx().unwrap();
                    if let Err(_) = tx.flush_tx() {
                        error!("Error flushing tx pipes");
                    }

                    // NOTE: Long delay to allow peer to change state
                    Timer::after(Duration::from_millis(50)).await;

                    info!("Sending response");
                    if let Err(_) = tx.send(val_received) {
                        error!("Error sending response");
                    }


                    loop {
                        Timer::after(Duration::from_micros(10)).await;
                        if let Ok(success) = tx.poll_send() {
                            match success {
                                true => {
                                    info!("Message sent successfully");
                                }
                                false => {
                                    error!("There was an issue sending the message");
                                },
                            }
                            break;
                        }
                    }

                    // NOTE: Switching to standby is stated to take up to 160us
                    rx = tx.standby().unwrap().rx().unwrap();
                    Timer::after(Duration::from_micros(160)).await;

                    led.set_low();
                }
            }
        }
    }
}
