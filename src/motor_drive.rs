use embassy_rp::gpio::Level;
use embassy_rp::pwm::Config;
use embassy_rp::peripherals::PIN_4;
use embassy_rp::peripherals::PWM_CH2;
use embassy_rp::pwm::Pwm;
use embassy_rp::peripherals::PIN_3;
use embassy_rp::peripherals::PIN_2;
use embassy_rp::gpio::Output;

pub struct MotorDrive<'a> {
    pub dir_1: Output<'a, PIN_2>,
    pub dir_2: Output<'a, PIN_3>,
    pub pwm: Pwm<'a, PWM_CH2>,
    pub target_speed: u8,
    pub config: Config
}

impl MotorDrive<'_> {
    pub fn new(
        pwm_pin: PIN_4,
        pwm_channel: PWM_CH2,
        dir_1_pin: PIN_2,
        dir_2_pin: PIN_3,
    ) -> MotorDrive<'static> {
        let mut config: Config = Default::default();
        config.top = 0x8000;

        MotorDrive {
            dir_1: Output::new(dir_1_pin, Level::Low),
            dir_2: Output::new(dir_2_pin, Level::Low),
            pwm: Pwm::new_output_a(pwm_channel, pwm_pin, config.clone()),
            target_speed: Default::default(),
            config
        }
    }

    pub fn set_forward_drive(&mut self) {
        self.dir_1.set_high();
        self.dir_2.set_low();
    }

    pub fn set_reverse_drive(&mut self) {
        self.dir_1.set_low();
        self.dir_2.set_high();
    }

    pub fn set_target_speed(&mut self, target: u8) {
        self.target_speed = target;
    }

    pub fn stop(&mut self) {
        self.set_target_speed(0);
    }

    fn set_motor_speed(&mut self) {
        self.config.compare_a = 10000;
        self.pwm.set_config(&self.config);
    }
}
