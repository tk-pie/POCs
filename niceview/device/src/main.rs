#![no_std]
#![no_main]
#![allow(unreachable_code)]

mod nice_view;

use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::gpio::{InputConfig, OutputConfig};
use esp_hal::spi::Mode;
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{
    clock::CpuClock,
    gpio::{Input, Level, Output, Pull},
    spi::{self, master::Spi},
};
use nice_view::NiceView;
use niceview_lib::{Icon, KeyboardDisplay, TextVariant};

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    esp_println::logger::init_logger(log::LevelFilter::Info);
    // esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timg0 = TimerGroup::new(peripherals.TIMG0);

    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    esp_alloc::heap_allocator!(size: 72 * 1024);

    log::warn!("WARN");
    log::error!("ERROR");
    log::debug!("DEBUG");
    log::trace!("TRACE");
    log::info!("INFO");

    log::info!("Press button to continue");
    let mut button = Input::new(
        peripherals.GPIO9,
        InputConfig::default().with_pull(Pull::Up),
    );
    button.wait_for_low().await;
    log::info!("Let's go!");

    // 22, 21,  20
    // CS, SCK, MOSI
    // Required SPI config for the NiceView
    let spi_config = spi::master::Config::default()
        .with_frequency(Rate::from_mhz(1))
        .with_mode(Mode::_0)
        .with_write_bit_order(spi::BitOrder::LsbFirst)
        .with_read_bit_order(spi::BitOrder::LsbFirst); // probably useless

    // NiceView cs is active high. I think...
    let cs = Output::new(peripherals.GPIO23, Level::Low, OutputConfig::default());

    let spi = Spi::new(peripherals.SPI2, spi_config)
        .expect("SPI config is valid")
        .with_sck(peripherals.GPIO22)
        .with_mosi(peripherals.GPIO21)
        .with_dma(peripherals.DMA_CH0)
        .into_async();

    let mut nice_view = NiceView::new(spi, cs);

    nice_view.clear_display().await;
    nice_view.fill_white();

    nice_view.draw_text("HELLO WORLD!", 10, 10, TextVariant::Regular);

    nice_view.draw_icon(Icon::BatteryFull, 40, 40);

    nice_view.flush().await;

    loop {
        nice_view.fill_white();
    }
}

#[repr(u8)]
#[allow(unused)]
enum GpioECommand {
    InputPort0 = 0,
    InputPort1 = 1,
    OuputPort0 = 2,
    OuputPort1 = 3,
    PolarityInversionPort0 = 4,
    PolarityInversionPort1 = 5,
    ConfigurationPort0 = 6,
    ConfigurationPort1 = 7,
}
