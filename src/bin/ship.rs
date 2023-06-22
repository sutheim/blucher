#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use common::motor_control::motor_control;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    spawner.spawn(motor_control(p.PIN_4, p.PWM_CH2, p.PIN_2, p.PIN_3)).unwrap();
}

