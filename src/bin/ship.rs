#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use common::{thruster::Thruster, locomotion_system::locomotion_system};
use defmt::unwrap;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let send_spawner = spawner.make_send();

    let thruster = Thruster::new(p.PIN_4, p.PWM_CH2, p.PIN_2.into(), p.PIN_3.into());
    unwrap!(spawner.spawn(locomotion_system(send_spawner, thruster)));
}

