#![no_std]
#![no_main]

use core::fmt::Write;
use defmt::info;
use embassy_executor::Spawner;
use embassy_nrf::{bind_interrupts, peripherals, twim, uarte};
use embassy_time::{Duration, Timer, with_timeout};
use heapless::String;
use static_cell::ConstStaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    SERIAL20 => twim::InterruptHandler<peripherals::SERIAL20>;
    SERIAL21 => uarte::InterruptHandler<peripherals::SERIAL21>;
});

const SI7021: u8 = 0x40;
const MEASURE_TEMP_NO_HOLD: u8 = 0xF3;
const MEASURE_HUMIDITY_NO_HOLD: u8 = 0xF5;
const I2C_TIMEOUT: Duration = Duration::from_millis(100);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    //I2C Setup

    let config = twim::Config::default();

    static RAM_BUFFER: ConstStaticCell<[u8; 16]> = ConstStaticCell::new([0; 16]);

    let mut i2c = twim::Twim::new(
        p.SERIAL20,
        Irqs,
        p.P1_12,
        p.P1_13,
        config,
        RAM_BUFFER.take(),
    );

    info!("i2c intialized");

    //UART Setup
    let mut config = uarte::Config::default();
    config.parity = uarte::Parity::Excluded;
    config.baudrate = uarte::Baudrate::Baud115200;

    let mut uart = uarte::Uarte::new(p.SERIAL21, p.P1_05, p.P1_04, Irqs, config);

    info!("uarte initilaized");

    loop {
        let mut temp_c: f32 = 0.0;
        let mut humidity: f32 = 0.0;

        match with_timeout(I2C_TIMEOUT, i2c.write(SI7021, &[MEASURE_TEMP_NO_HOLD])).await {
            Ok(Ok(())) => {
                Timer::after_millis(25).await;

                let mut buffer = [0u8; 2];

                match with_timeout(I2C_TIMEOUT, i2c.read(SI7021, &mut buffer)).await {
                    Ok(Ok(())) => {
                        let raw = u16::from_be_bytes(buffer) >> 1;
                        temp_c = ((175.72 * raw as f32) / 65536.0) - 46.85;
                    }
                    Ok(Err(e)) => info!(" temp read error: {:?}", e),

                    Err(_) => info!("temp read timed out"),
                }
            }

            Ok(Err(e)) => info!("temp write error: {:?}", e),

            Err(_) => info!("temp write timed out"),
        }

        match with_timeout(I2C_TIMEOUT, i2c.write(SI7021, &[MEASURE_HUMIDITY_NO_HOLD])).await {
            Ok(Ok(())) => {
                Timer::after_millis(25).await;
                let mut buffer = [0u8; 2];

                match with_timeout(I2C_TIMEOUT, i2c.read(SI7021, &mut buffer)).await {
                    Ok(Ok(())) => {
                        let raw_h = u16::from_be_bytes(buffer);
                        humidity = (125.0 * raw_h as f32) / 65536.0 - 6.0;
                    }
                    Ok(Err(e)) => info!("humidity read error: {:?}", e),

                    Err(_) => info! {"humidity read timed out"},
                }
            }
            Ok(Err(e)) => info!("humidity write error: {:?}", e),

            Err(_) => info!("humidity write timed out"),
        }

        let mut msg: String<64> = String::new();
        let _ = write!(msg, "temp: {:.2} C, humidity: {:.2}%\r\n", temp_c, humidity);
        let _ = uart.write(msg.as_bytes()).await;
        info!("temp: {} C, humidity: {}%", temp_c, humidity);

        Timer::after_millis(1000).await;
    }
}
