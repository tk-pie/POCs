//! A bluetooth battery service example built using Embassy and trouBLE.

//% CHIP_FILTER: bt_driver_supported

#![no_std]
#![no_main]

mod ble_hid_kbm;
mod hid_report;

use ble_hid_kbm::KEY_EVENTS;
use embassy_executor::Spawner;
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    gpio::{AnyPin, Input, InputConfig, Pin},
    timer::timg::TimerGroup,
};
use esp_radio::ble::controller::BleConnector;
use log::info;
use trouble_host::prelude::*;

use crate::hid_report::HID_DESCRIPTOR;

use {embassy_crypto_rustcrypto as _, esp_alloc as _, esp_backtrace as _};

esp_bootloader_esp_idf::esp_app_desc!();

#[embassy_executor::task]
async fn boot_pin_watcher(pin: AnyPin<'static>) {
    let mut input = Input::new(pin, InputConfig::default());
    loop {
        input.wait_for_low().await;
        info!("[key] pressed");
        KEY_EVENTS.send(true).await;
        input.wait_for_high().await;
        info!("[key] released");
        KEY_EVENTS.send(false).await;
    }
}

#[esp_hal::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();
    log::info!("{:?}", HID_DESCRIPTOR);
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    esp_alloc::heap_allocator!(size: 72 * 1024);
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    spawner.spawn(boot_pin_watcher(peripherals.GPIO9.degrade()).unwrap());

    let bluetooth = peripherals.BT;
    let connector = BleConnector::new(bluetooth, Default::default()).unwrap();
    let controller: ExternalController<_, 20> = ExternalController::new(connector);

    ble_hid_kbm::run(controller).await;
}
