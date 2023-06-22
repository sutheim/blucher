use defmt::info;
use embassy_rp::peripherals::{PIN_2, PIN_3, PIN_4, PWM_CH2};
use embassy_time::{Duration, Timer};

use crate::motor_drive::MotorDrive;
use {defmt_rtt as _, panic_probe as _};


#[embassy_executor::task]
pub async fn motor_control(
    pwm_pin: PIN_4,
    pwm_channel: PWM_CH2,
    dir_1_pin: PIN_2,
    dir_2_pin: PIN_3,
) {
    info!("Initializing Motor Control Systems");

    let mut drive = MotorDrive::new(pwm_pin, pwm_channel, dir_1_pin, dir_2_pin);

    loop {
        drive.set_forward_drive();

        Timer::after(Duration::from_millis(100)).await;
    }
}
