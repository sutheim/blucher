use defmt::info;
use embassy_rp::{
    gpio::{Level, Output},
    peripherals::{PIN_2, PIN_3, PIN_4, PWM_CH2},
    pwm::{Config, Pwm},
};
use embassy_time::{Duration, Instant, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task]
pub async fn motor_control(
    pwm_pin: PIN_4,
    pwm_channel: PWM_CH2,
    dir_1_pin: PIN_2,
    dir_2_pin: PIN_3,
) {
    let mut c: Config = Default::default();
    c.top = 0x8000;
    c.compare_a = 0;

    let mut pwm = Pwm::new_output_a(pwm_channel, pwm_pin, c.clone());

    let mut motor_dir_1 = Output::new(dir_1_pin, Level::High);
    let mut motor_dir_2 = Output::new(dir_2_pin, Level::High);

    loop {
        info!("current LED duty cycle: {}/32768", c.compare_a);
        let time = Instant::now().as_millis() % (32768 - 10000) + 10000;
        let time = time as u16;
        Timer::after(Duration::from_millis(100)).await;
        c.compare_a = time;
        pwm.set_config(&c);
    }
}
