# Use Ubuntu 22.04 as base image (supported for TDX on both GCP and Azure)
FROM ubuntu:22.04

# Set environment variables to avoid interactive prompts during installation
ENV DEBIAN_FRONTEND=noninteractive

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libtss2-dev \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install Rust and set to nightly
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly
ENV PATH="/root/.cargo/bin:${PATH}"

# The TDX device will be mounted at /sys/kernel/config/tsm/report at runtime

# Set working directory
WORKDIR /app

# Copy the project files
COPY . .

# Build the attestation and fmspc example
RUN cargo build --example attestation
RUN cargo build --example fmspc

# Set the entrypoint to run the fmspc example
ENTRYPOINT ["./target/debug/examples/fmspc"]
