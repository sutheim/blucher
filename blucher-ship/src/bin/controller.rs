#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::u8;

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio;
use embassy_time::{Duration, Timer};
use gpio::{Level, Output};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialise Peripherals
    let p = embassy_rp::init(Default::default());

    // Create LED
    let mut led = Output::new(p.PIN_25, Level::Low);

    // Loop
    loop {

        led.set_high();
        info!("On");
        Timer::after(Duration::from_millis(1000)).await;

        led.set_low();

        info!("Off");
        Timer::after(Duration::from_millis(1000)).await;
    }
}
