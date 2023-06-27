use half::f16;
use embassy_rp::{
    gpio::{Level, Output, AnyPin},
    peripherals::{PIN_4, PWM_CH2},
    pwm::{Config, Pwm},
};

const EPSILON : f32 = 0.001;

pub struct Thruster {
    pub dir_1: Output<'static, AnyPin>,
    pub dir_2: Output<'static, AnyPin>,
    pub pwm: Pwm<'static, PWM_CH2>,

    pub target_drive: f16,
    pub current_drive: f16,
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
        }
    }

    pub fn is_target_drive_reached(&self) -> bool {
        let mut val = f32::from(self.target_drive - self.current_drive);
        if val < 0f32
        {
            val *= -1f32;
        }

        val < EPSILON
    }

    pub fn set_target_drive(&mut self, target: f32) {
        self.target_drive = f16::from_f32(target);
    }

    pub fn stop(&mut self) {
        self.target_drive = f16::from_f32_const(0f32);
    }

    fn set_forward_drive(&mut self) {
        self.dir_1.set_high();
        self.dir_2.set_low();
    }

    fn set_reverse_drive(&mut self) {
        self.dir_1.set_low();
        self.dir_2.set_high();
    }

    fn set_motor_speed(&mut self) {
        self.config.compare_a = 10000;
        self.pwm.set_config(&self.config);
    }
}
