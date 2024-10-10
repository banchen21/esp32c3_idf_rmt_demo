use std::time::SystemTime;

use anyhow::Result;
use chrono::{DateTime, Utc};
use esp32c3_wifi::wifi;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::prelude::Peripherals,
    sntp::{EspSntp, SyncStatus},
    wifi::AuthMethod,
};
use log::info;

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;

    let ssid = "test";
    let pass = "";
    let auth_method = AuthMethod::None;
    let _wifi = wifi(ssid, pass, auth_method, peripherals.modem, sysloop)?;

    let ntp = EspSntp::new_default().unwrap();
    // Synchronize NTP
    println!("Synchronizing with NTP Server");
    while ntp.get_sync_status() != SyncStatus::Completed {}
    println!("Time Sync Completed");

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        // Obtain System Time
        let st_now = SystemTime::now();
        // Convert to UTC Time
        let dt_now_utc: DateTime<Utc> = st_now.clone().into();
        // Format Time String
        let formatted = format!("{}", dt_now_utc.format("%d/%m/%Y %H:%M:%S"));
        info!("{formatted}");
    }
}
