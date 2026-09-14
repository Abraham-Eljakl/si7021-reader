# si7021-reader

Embassy firmware for thr nRF54L15 devkit that reads temperature and humidity from  Si7021 sensor over I2C and forwards that reading over UART, for display in a terminal.

## Status

LED Blink and push button verified to work

## Getting started

This project uses Nix flake to provide a reproducible set of tools to manage the Rust complier and cross-compliation target.

1. Enter the dev enviroment: nix develop

2. Build and flash (first flash on a fresh/locked chip needs an erase permission): cargo run --release -- --allow-erase-all

3. Subsequent flahses don't need that flag: cargo run --release

