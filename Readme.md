# Get Started Guide (for Linux)

This guide will help you set up your Linux environment for developing with the ESP32 microcontroller and rust.

## Install ESP32 Prerequisites

Install the required packages for working with the ESP32 on Ubuntu/Debian-based Linux distributions.

```bash
sudo apt-get install git wget flex bison gperf python3 python3-pip python3-venv cmake ninja-build ccache libffi-dev libssl-dev dfu-util libusb-1.0-0 libudev-dev
```

## Install Rust and cargo-binstall

### Rust

Install the Rust programming language, which is required for developing with the ESP32.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Cargo-binstall

Install cargo-binstall, a tool for installing the binaries directly instead of compiling them This will reduce the setup time immensely.

```bash
curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
```

> Note: If you encounter any issues or get stuck for more than 30 seconds during the installation process, try running the command using `cargo install` instead of `cargo-binstall`.

## Install ESP-IDF Prerequisites

Install the necessary tools and utilities for working with the Espressif IoT Development Framework (ESP-IDF).

``` bash
cargo-binstall cargo-generate
cargo-binstall ldproxy
cargo-binstall espup
cargo-binstall espflash
cargo-binstall cargo-espflash # Optional
```

## Install toolchaings for Espressif
``` bash
espup install
. $HOME/export-esp.sh
```

## Build the rust binary
To create the esp32 executeable run 
``` bash
cargo build
```
The binary is saved in the following directory: `target/xtensa-esp32-espidf/debug/lock-example`

## To flash the newly made binary run
``` bash
espflash flash target/xtensa-esp32-espidf/debug/lock-example
```

## To monitor the run
``` bash
espflash monitor
```
