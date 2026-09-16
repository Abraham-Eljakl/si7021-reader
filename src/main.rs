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

//Binding interrupts for the two serial periphrials in use:
//Needed one for I2C, Serial 20, and one for UART, Serial 21
bind_interrupts!(struct Irqs {
    SERIAL20 => twim::InterruptHandler<peripherals::SERIAL20>;
    SERIAL21 => uarte::InterruptHandler<peripherals::SERIAL21>;
});

const SI7021: u8 = 0x40;
const MEASURE_TEMP_NO_HOLD: u8 = 0xF3;
const MEASURE_HUMIDITY_NO_HOLD: u8 = 0xF5;
//Chosen to be generous enough to cover a normal transaction, tight enough to recover
////quicly from a locked up buss rather than hang forever
const I2C_TIMEOUT: Duration = Duration::from_millis(100);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    //I2C Setup
    let config = twim::Config::default();

    //twim data can only accesss ram, not flash it, so the transfer buffer must be static within the
    //ram so the twim easyDMA can accesses it, ram allocation rather than a plain array is done because
    //of that so on flash it is still able to access the buffer apposed to a static array that is in
    //the memory allocation that gets flashed on a compile
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

    //UART Setup: forwards readings to laptops iver Serial21
    //where p1.04 and p1.05 are the boards vcom1 pins, with baud rates set at 115200
    let mut config = uarte::Config::default();
    config.parity = uarte::Parity::Excluded;
    config.baudrate = uarte::Baudrate::Baud115200;

    let mut uart = uarte::Uarte::new(p.SERIAL21, p.P1_05, p.P1_04, Irqs, config);

    info!("uarte initilaized");

    loop {
        let mut temp_c: f32 = 0.0;
        let mut humidity: f32 = 0.0;
        //tracks whether every step this cycle succeded, this was put in as bus lock up was seen
        //between measurements so using with_timeout to ensure when a lock up occures, instead of
        //locking out rhe bus line it will skip the write or read and prevent any misleading data
        //from going over UART
        let mut ok = true;
        let mut msg: String<64> = String::new();

        //no hold master was chosen so that the sensor never stretched the clcok so we can
        //explicitly delay instead of relying on a blocking combined
        //write_read, this plaes nices with the async executor so other tasks can run during the
        //wait
        match with_timeout(I2C_TIMEOUT, i2c.write(SI7021, &[MEASURE_TEMP_NO_HOLD])).await {
            Ok(Ok(())) => {
                Timer::after_millis(25).await;

                let mut buffer = [0u8; 2];

                match with_timeout(I2C_TIMEOUT, i2c.read(SI7021, &mut buffer)).await {
                    Ok(Ok(())) => {
                        let raw = u16::from_be_bytes(buffer) >> 1;
                        //right shifted by 1 to correct a bit-slip that is seen, comfirmed through
                        //testing that this is a signal integirty quirk, not a formula or wiring
                        //error
                        temp_c = ((175.72 * raw as f32) / 65536.0) - 46.85;
                    }
                    Ok(Err(e)) => {
                        let _ = write!(msg, "temp read error: {:?}%\r\n", e);
                        let _ = uart.write(msg.as_bytes()).await;
                        info!(" temp read error: {:?}", e);
                        //a i2c error occured: mark the cycle as bad so the stale temp_c value never
                        //gets sent
                        ok = false;
                    }
                    Err(_) => {
                        let _ = write!(msg, "temp read timed out\r\n");
                        let _ = uart.write(msg.as_bytes()).await;
                        info!("temp read timed out");
                        //bus never responded within a resonable and timed out so for the same
                        //reason it wil not report the reading that never occured
                        ok = false;
                    }
                }
            }

            Ok(Err(e)) => {
                let _ = write!(msg, "temp write error: {:?}%\r\n", e);
                let _ = uart.write(msg.as_bytes()).await;
                info!("temp write error: {:?}", e);
                //same logic seen in the temp read but for writing on the bus
                ok = false;
            }

            Err(_) => {
                let _ = write!(msg, "temp write timed out\r\n");
                let _ = uart.write(msg.as_bytes()).await;
                info!("temp write timed out");
                //same logic as seen in temp read but for writing and the bus times out
                ok = false;
            }
        }

        match with_timeout(I2C_TIMEOUT, i2c.write(SI7021, &[MEASURE_HUMIDITY_NO_HOLD])).await {
            Ok(Ok(())) => {
                Timer::after_millis(25).await;
                let mut buffer = [0u8; 2];

                match with_timeout(I2C_TIMEOUT, i2c.read(SI7021, &mut buffer)).await {
                    Ok(Ok(())) => {
                        let raw_h = u16::from_be_bytes(buffer);
                        //no shift here, unlike temp, it was tested and confirmed that the humidity
                        //reading is correct, the bit slip appears only for the temo reading
                        humidity = (125.0 * raw_h as f32) / 65536.0 - 6.0;
                    }
                    Ok(Err(e)) => {
                        let _ = write!(msg, "humidity read error: {:?}\r\n", e);
                        let _ = uart.write(msg.as_bytes()).await;
                        info!("humidity read error: {:?}", e);
                        ok = false;
                    }

                    Err(_) => {
                        let _ = write!(msg, "humidity read timed out%\r\n");
                        let _ = uart.write(msg.as_bytes()).await;
                        info! {"humidity read timed out"};
                        ok = false;
                    }
                }
            }
            Ok(Err(e)) => {
                let _ = write!(msg, "humidity write error: {:?}%\r\n", e);
                let _ = uart.write(msg.as_bytes()).await;
                info!("humidity write error: {:?}", e);
                ok = false;
            }

            Err(_) => {
                let _ = write!(msg, "humidity write timed out\r\n");
                let _ = uart.write(msg.as_bytes()).await;
                info!("humidity write timed out");
                ok = false;
            }
        }

        //forward the reading to the laptop over uart and log it locally if both are genuinely
        //succeded during the cycle, this wil prevent a partial or fully failed cycle from reporting zeros as if it were real sensor data
        if ok {
            let _ = write!(msg, "temp: {:.2} C, humidity: {:.2}%\r\n", temp_c, humidity);
            let _ = uart.write(msg.as_bytes()).await;
            info!("temp: {} C, humidity: {}%", temp_c, humidity);
        }

        Timer::after_millis(1000).await;
    }
}
