use embassy_rp::{
    gpio::{AnyPin, Level, Output},
    peripherals::{PIN_4, PWM_CH2},
    pwm::{Config, Pwm},
};
use half::f16;

use defmt::debug;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex, signal::Signal};
use embassy_time::{Duration, Instant, Timer};

use {defmt_rtt as _, panic_probe as _};

use blucher_core::util::smooth_damp;

const THRUSTER_TICK_INTERVAL: u8 = 50;
const EPSILON: f32 = 0.01;

pub static TARGET_THRUSTER_DRIVE: Signal<ThreadModeRawMutex, f32> = Signal::new();
// pub static TARGET_THRUSTER_DRIVE: Mutex<ThreadModeRawMutex, f32> = Mutex::new(0f32);

pub struct Thruster {
    pub dir_1: Output<'static, AnyPin>,
    pub dir_2: Output<'static, AnyPin>,
    pub pwm: Pwm<'static, PWM_CH2>,

    pub target_drive: f16,
    pub current_drive: f16,
    pub current_drive_velocity: f16,
    pub config: Config,
}

impl Thruster {
    pub fn new(
        pwm_pin: PIN_4,
        pwm_channel: PWM_CH2,
        dir_1_pin: AnyPin,
        dir_2_pin: AnyPin,
    ) -> Thruster {
        let mut config: Config = Default::default();
        config.top = 0x8000;

        Thruster {
            dir_1: Output::new(dir_1_pin, Level::Low),
            dir_2: Output::new(dir_2_pin, Level::Low),
            pwm: Pwm::new_output_a(pwm_channel, pwm_pin, config.clone()),
            target_drive: f16::from_f32_const(0f32),
            current_drive: f16::from_f32_const(0f32),
            config,
            current_drive_velocity: f16::from_f32_const(0f32),
        }
    }

    pub fn is_target_drive_reached(&mut self) -> bool {
        let mut val = f32::from(self.target_drive - self.current_drive);
        if val < 0f32 {
            val = -val;
        }

        let reached = val < EPSILON;

        if reached {
            self.current_drive = self.target_drive;
        }

        reached
    }

    pub fn set_target_drive(&mut self, target: f32) {
        let value = f16::from_f32(target);
        let value = f16::max(value, f16::from_f32_const(-1f32));
        let value = f16::min(value, f16::from_f32_const(1f32));

        self.target_drive = value;
    }

    pub fn tick_thruster(&mut self, delta_t_micros: u64) {
        let dt = TryInto::<u32>::try_into(delta_t_micros).unwrap() as f32 / 100_000f32;

        if self.current_drive > f16::from_f32_const(-0.01f32) && self.current_drive < f16::from_f32_const(0.01f32) {
            if self.target_drive > f16::from_f32_const(0f32) {
                self.current_drive = f16::from_f32_const(0.25);
            }
            else {
                self.current_drive = f16::from_f32_const(-0.25);
            }
        }

        let old = self.current_drive;

        self.current_drive = f16::from_f32(smooth_damp(
            self.current_drive.into(),
            self.target_drive.into(),
            &mut self.current_drive_velocity.into(),
            1.5f32,
            dt,
        ));

        self.set_motor_speed_according_to_drive();
    }

    fn set_forward_drive(&mut self) {
        self.dir_1.set_high();
        self.dir_2.set_low();
    }

    fn set_reverse_drive(&mut self) {
        self.dir_1.set_low();
        self.dir_2.set_high();
    }

    fn update_drive_direction(&mut self) {
        if self.current_drive.to_f32() > 0f32 {
            self.set_forward_drive();
        } else {
            self.set_reverse_drive();
        }
    }

    fn set_motor_speed_according_to_drive(&mut self) {
        self.update_drive_direction();

        let drive = self.current_drive.to_f32();

        let throttle = if drive > 0f32 {
            drive
        } else {
            -drive
        };

        // self.config.compare_a = (throttle * 32768f32) as u16;
        self.config.compare_a = (throttle * 65_535f32) as u16;
        self.pwm.set_config(&self.config);

        debug!("Duty cycle set to: {}", self.config.compare_a);
    }
}

#[embassy_executor::task]
pub async fn tick_thruster_task(
    mut thruster: Thruster,
) {
    let mut last_tick = Instant::now();
    let mut target = 0f32;

    loop {
        if thruster.is_target_drive_reached() {
            debug!("Thruster drive target reached, waiting.");

            target = TARGET_THRUSTER_DRIVE.wait().await;

            last_tick = Instant::now() - Duration::from_millis(THRUSTER_TICK_INTERVAL.into());

            debug!("Thruster drive target changed, engaging thruster control");
        }
        else {
            if TARGET_THRUSTER_DRIVE.signaled() {
                target = TARGET_THRUSTER_DRIVE.wait().await;
            }
        }

        thruster.set_target_drive(target);

        let delta_t = ((Instant::now() - last_tick) as Duration).as_micros();

        thruster.tick_thruster(delta_t);

        last_tick = Instant::now();

        Timer::after(Duration::from_millis(THRUSTER_TICK_INTERVAL.into())).await;
    }
}
