use embassy_rp::{i2c::{I2c, Async}, peripherals::I2C1};
use embassy_time::{Timer, Duration};


const COMMS_I2C_TICK_INTERVAL: u8 = 5;

#[embassy_executor::task]
pub async fn handle_radio_task (
    _i2c: I2c<'static, I2C1, Async>
) {
    loop {
        Timer::after(Duration::from_millis(COMMS_I2C_TICK_INTERVAL.into())).await;
    }
}
