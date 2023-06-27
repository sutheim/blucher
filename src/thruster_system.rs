use defmt::{info, debug};
use embassy_executor::SendSpawner;
use embassy_time::{Duration, Timer};
use embassy_sync::{signal::Signal, blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};

use crate::thruster::Thruster;

use {defmt_rtt as _, panic_probe as _};

static DRIVE_UPDATED: Signal<ThreadModeRawMutex, ()> = Signal::new();
static TARGET_DRIVE: Mutex<ThreadModeRawMutex, f32> = Mutex::new(0f32);

#[embassy_executor::task]
pub async fn thruster_system (
    spawner: SendSpawner,
    thruster: Thruster
    ) {
    info!("Initializing Thruster Control Systems");

    spawner.spawn(update_thruster_task(thruster)).unwrap();

    loop {
        Timer::after(Duration::from_millis(100)).await;
    }
}


#[embassy_executor::task]
async fn update_thruster_task (
    mut thruster: Thruster
) {
    loop {
        if thruster.is_target_drive_reached() {
            debug!("Thruster drive target reached, waiting...");
            DRIVE_UPDATED.wait().await;
            debug!("Thruster drive target changed, engaging");
        }
        thruster.set_target_drive(*TARGET_DRIVE.lock().await);

        Timer::after(Duration::from_millis(10)).await;
    }
}
