#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(async_fn_in_trait)]
#![allow(incomplete_features)]
#![allow(dead_code)]

pub mod locomotion_system;
pub mod radio;
pub mod thruster;
pub mod wifi_system;

use blucher_data::commands::Command;
use blucher_data::wifi::SHIP_PASSPHRASE;
use blucher_data::wifi::SHIP_SSID;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;

use crate::{
    // locomotion_system::locomotion_system, thruster::Thruster,
    wifi_system::wifi_system,
};

use defmt::unwrap;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

static COMMAND_CHANNEL: Channel<ThreadModeRawMutex, Command, 8> = Channel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // let send_spawner = spawner.make_send();

    unwrap!(spawner.spawn(wifi_system(
        spawner,
        SHIP_SSID,
        SHIP_PASSPHRASE,
        p.PIN_23,
        p.PIN_25,
        p.PIO0,
        p.PIN_24,
        p.PIN_29,
        p.DMA_CH0,
        COMMAND_CHANNEL.sender(),
    )));

    // let thruster = Thruster::new(p.PIN_4, p.PWM_CH2, p.PIN_2.into(), p.PIN_3.into());
    // unwrap!(spawner.spawn(locomotion_system(spawner, thruster)));
}


