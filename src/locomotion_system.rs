use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

use crate::thruster::{Thruster, tick_thruster_task, TARGET_THRUSTER_DRIVE};

use {defmt_rtt as _, panic_probe as _};


#[embassy_executor::task]
pub async fn locomotion_system (
    spawner: Spawner,
    thruster: Thruster
    //TODO: add stepper for rudder
    ) {
    info!("Initializing Thruster Control Systems");

    unwrap!(spawner.spawn(tick_thruster_task(
        thruster
    )));

    //TODO: spawn rudder control task

    loop {
        // TODO: query main direction and speed instructions
        Timer::after(Duration::from_millis(2000)).await;
        {
            TARGET_THRUSTER_DRIVE.signal(0.4);
        }
        Timer::after(Duration::from_millis(2000)).await;
        {
            TARGET_THRUSTER_DRIVE.signal(0.0);
        }
        Timer::after(Duration::from_millis(2000)).await;
        {
            TARGET_THRUSTER_DRIVE.signal(-0.4);
        }
        Timer::after(Duration::from_millis(2000)).await;
        {
            TARGET_THRUSTER_DRIVE.signal(0.0);
        }
    }
}


