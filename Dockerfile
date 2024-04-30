FROM ubuntu:22.04

# Install required packages
RUN apt-get update && apt-get install -y \
    git \
    wget \
    flex \
    bison \
    gperf \
    python3 \
    python3-pip \
    python3-venv \
    cmake \
    ninja-build \
    ccache \
    libffi-dev \
    libssl-dev \
    dfu-util \
    libusb-1.0-0 \
    libudev-dev \
    curl \ 
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Update environment variables for Rust
ENV PATH="/root/.cargo/bin:${PATH}"

# Install cargo-binstall
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

# Install ESP-IDF Prerequisites
RUN cargo-binstall -y \
    cargo-generate \
    ldproxy \
    espup \
    espflash \
    cargo-espflash

# Install toolchains for Espressif
RUN espup install && \
    . $HOME/export-esp.sh

# Set the working directory
WORKDIR /app

# Copy the project files
COPY . .

# Build the Rust binary
RUN cargo build

# Flash and monitor the binary (simulated)
RUN echo "Simulating espflash flash target/xtensa-esp32-espidf/debug/assignment2 -M"

CMD ["bash"]