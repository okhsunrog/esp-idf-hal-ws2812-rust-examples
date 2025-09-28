use esp_idf_svc::hal::prelude::*;
use smart_leds::{brightness, SmartLedsWrite, RGB8};
use ws2812_esp32_rmt_driver::Ws2812Esp32Rmt;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    log::info!("WS2812 RMT (blocking) example");

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    let led_pin = pins.gpio10;
    let mut ws = Ws2812Esp32Rmt::new(peripherals.rmt.channel0, led_pin)?;

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
