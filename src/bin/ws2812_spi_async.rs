use embassy_executor::Executor;
use embassy_time::Timer;
use esp_idf_svc::hal::{
    gpio::AnyInputPin,
    prelude::*,
    spi::{config::Config as SpiConfig, Dma, SpiBusDriver, SpiDriver, SpiDriverConfig},
    units::Hertz,
};
use smart_leds::{brightness, SmartLedsWriteAsync, RGB8};
use static_cell::StaticCell;
use std::time::Duration;
use ws2812_async::{Grb, Ws2812};

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[embassy_executor::task]
async fn rainbow_task() {
    // Initialize SPI for WS2812
    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    // ESP32-C3-Zero WS2812: data on GPIO10 via MOSI. No SCLK line connected.
    let mosi = pins.gpio10;
    let bus_cfg = SpiDriverConfig::new().dma(Dma::Auto(1024));
    let miso: Option<AnyInputPin> = None;
    let spi_bus = SpiDriver::new_without_sclk(peripherals.spi2, mosi, miso, &bus_cfg)
        .expect("SPI bus init failed");

    // WS2812 expects ~800kHz bitstream; SPI encoding library maps bits, so use 3.2MHz
    let spi_cfg = SpiConfig::new().baudrate(Hertz(3_200_000)).write_only(true);
    let mut spi_dev = SpiBusDriver::new(spi_bus, &spi_cfg).expect("SPI device init failed");

    // IMPORTANT! Need to set CONFIG_SPI_MASTER_ISR_IN_IRAM=n in sdkconfig.defaults for async spi support in esp-idf-hal
    let mut ws: Ws2812<_, Grb, { 12 * NUM_LEDS }> = Ws2812::new(&mut spi_dev);

    const NUM_LEDS: usize = 1;
    let mut data = [RGB8::default(); NUM_LEDS];

    loop {
        for j in 0..(256 * 5) {
            for (i, item) in data.iter_mut().enumerate().take(NUM_LEDS) {
                // Generate rainbow colors using wheel function
                let mut wheel_pos = (((i * 256) as u16 / NUM_LEDS as u16 + j as u16) & 255) as u8;
                wheel_pos = 255 - wheel_pos;

                *item = if wheel_pos < 85 {
                    RGB8::new(255 - wheel_pos * 3, 0, wheel_pos * 3)
                } else if wheel_pos < 170 {
                    wheel_pos -= 85;
                    RGB8::new(0, wheel_pos * 3, 255 - wheel_pos * 3)
                } else {
                    wheel_pos -= 170;
                    RGB8::new(wheel_pos * 3, 255 - wheel_pos * 3, 0)
                };
            }
            ws.write(brightness(data.iter().cloned(), 32))
                .await
                .unwrap();
            Timer::after_millis(10).await;
        }
    }
}

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    log::info!("WS2812 SPI (async) example");

    std::thread::Builder::new()
        .stack_size(10000)
        .spawn(|| {
            let executor = EXECUTOR.init(Executor::new());
            executor.run(|spawner| {
                spawner.spawn(rainbow_task()).unwrap();
            });
        })
        .unwrap();

    loop {
        std::thread::sleep(Duration::from_secs(10));
        std::thread::yield_now();
    }
}
