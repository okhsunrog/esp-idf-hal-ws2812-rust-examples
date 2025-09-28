use esp_idf_svc::hal::{
    gpio::AnyInputPin,
    prelude::*,
    spi::{config::Config as SpiConfig, Dma, SpiBusDriver, SpiDriver, SpiDriverConfig},
    units::Hertz,
};
use smart_leds::{brightness, SmartLedsWrite, RGB8};
use ws2812_spi::Ws2812;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    log::info!("WS2812 SPI (blocking) example");

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    // ESP32-C3-Zero WS2812: data on GPIO10 via MOSI. No SCLK line connected.
    let mosi = pins.gpio10;
    let bus_cfg = SpiDriverConfig::new().dma(Dma::Auto(4096));
    let miso: Option<AnyInputPin> = None;
    let spi_bus = SpiDriver::new_without_sclk(peripherals.spi2, mosi, miso, &bus_cfg)?;

    // WS2812 expects ~800kHz bitstream; SPI encoding library maps bits, so use 3.2MHz
    let spi_cfg = SpiConfig::new().baudrate(Hertz(3_200_000)).write_only(true);
    let spi_dev = SpiBusDriver::new(spi_bus, &spi_cfg)?;

    let mut ws = Ws2812::new(spi_dev);

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
            ws.write(brightness(data.iter().cloned(), 32))?;
            std::thread::sleep(std::time::Duration::from_millis(10));

            // Yield to allow other tasks to run
            std::thread::yield_now();
        }
    }
}
