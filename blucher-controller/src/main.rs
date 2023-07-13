#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(async_fn_in_trait)]
#![allow(incomplete_features)]
#![allow(dead_code)]

pub use blucher_core as core;
pub mod radio;
pub mod wifi_system;

use crate::wifi_system::wifi_system;
use defmt::unwrap;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // let send_spawner = spawner.make_send();

    unwrap!(spawner.spawn(wifi_system(
       spawner, p.PIN_23, p.PIN_25, p.PIO0, p.PIN_24, p.PIN_29, p.DMA_CH0
    )));
}
