use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::{i2c::{self, Config, InterruptHandler}, peripherals::{I2C1, PIN_14, PIN_15}, bind_interrupts};
use embassy_time::{Timer, Duration};

bind_interrupts!(struct Irqs {
    I2C1_IRQ => InterruptHandler<I2C1>;
});

const COMMS_TICK_INTERVAL: u8 = 5;

#[embassy_executor::task]
pub async fn radio_system (
    _spawner: Spawner,
    i2c1: I2C1,
    scl: PIN_15,
    sda: PIN_14,
    ) {

    info!("Initializing Comms Systems");

    let _i2c = i2c::I2c::new_async(i2c1, scl, sda, Irqs, Config::default());

    // let token = handle_radio_task(i2c);
    //
    // unwrap!(spawner.spawn(token));

    loop {
        Timer::after(Duration::from_millis(COMMS_TICK_INTERVAL.into())).await;
    }
}

