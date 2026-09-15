# si7021-reader

Embassy firmware for thr nRF54L15 devkit that reads temperature and humidity from  Si7021 sensor over I2C and forwards that reading over UART, for display in a terminal.

## Status

Fully working: reads temperature and humidity from a Si7021 over I2C, forwards both readings over UART to a connected terminal once per second. Verified end to end on hardware

## Getting started

This project uses Nix flake to provide a reproducible set of tools to manage the Rust complier and cross-compliation target.

1. Enter the dev enviroment: nix develop

2. Build and flash (first flash on a fresh/locked chip needs an erase permission): cargo run --release -- --allow-erase-all

3. Subsequent flahses don't need that flag: cargo run --release

4. Enter minicom: minicom -D /dev/ttyACMX -b 115200 (replace x with the correct port number)

## Design Decisions

1. Trust Zone mode: Secure vs Non-Secure. The nRF54L15 uses ARM TrustZone where running Non-Secure needs a seperate secure world firmware (e.g TF-M) to already be running and grant memory/peripheral access. Since this project doesn't have firmware like that, secure mode avoids that entirely.

2. memory.x: Reserves all 256K of ram to the application core. Usually half is reserved for the RISC-V coproccessor but this project doesnt use that so all RAM allocation is used

3. I2C: No Hold Master Mode was sued with the commans 0xF3 and 0xF5 rather than Hold Master Mode to avoud clock stretching an blocking the I2C bus during measurment. It also paired nicely with the async strucutre in embassy. 

4. embassy-nrf: Version used was 011 opposed to the project templeate of 0.9 to stay up to date with what is currently being used

5. I2C Reads are right shifted byt 1 bit to correct a bit slip seen when testing the sensor intially. 

6. All I2C operations are wrapped in a withj_timeout to recover from any I2C bus lockups rather than hanging

7. UART used P1.05 as RX and P1.04 as tx matching the Serial Port 1 pins.

## Investigation: why temp reads need a bit shift left by one

The raw temp value from the Si7021 consistently comes back as almot exactly double what the datasheet forula expects. I confirmed that the sensor was live and I2C was working with a simple touch to the sensor. Humidity showed no problems, only temperature was giving me issues over the bus line

Hyptheses tested and ruled out:

1. Under-voltage - Board configuratoe showed Vdd at 2.9 V rather than 3.3V, moved power input from VDDIO to VBUS to allow for more regulator head room but no changes were seen.

2. Pull-up strength - enabled the chip's internal SCL/SDA pull-ups in addition to the breakout board's external ones but no change was made to the raw byte data.

3. Sequencing / idle timing - swapping which measurment (temp or humidity) ran first in the loop did move the shift; it stayed attached to temperature specifically, regardless of position.

4. Command specificity - reading temperature via a completely different command ('OxEO', reading a value already computed during the prior humidity conversion) showed the same shift as the direct 0xF3 command

5. Sensor resolution/config - read the Si7021's user register directly and confirmed it matched the datasheet default exactly (00111010), ruling out a non-standard setting

Conclusion: The cause was due to a hardware issue, after swapping out with a different Si7021 the board began to read values correctly.
