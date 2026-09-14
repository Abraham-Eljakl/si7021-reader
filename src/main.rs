#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Input, Level, Output, OutputDrive, Pull};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task]
async fn blink_task(mut led: Output<'static>) {
    loop {
        led.set_high();
        Timer::after_millis(500).await;
        led.set_low();
        Timer::after_millis(500).await;
    }
}

#[embassy_executor::task]
async fn button_task(mut button: Input<'static>) {
    loop {
        button.wait_for_low().await;
        info!("Button pressed!");
        Timer::after_millis(50).await;

        button.wait_for_high().await;
        info!("Button released!");
        Timer::after_millis(50).await;
    }
}
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    let led = Output::new(p.P2_09, Level::Low, OutputDrive::Standard);
    let button = Input::new(p.P1_13, Pull::Up);

    spawner.spawn(blink_task(led).unwrap());
    spawner.spawn(button_task(button).unwrap());
}
