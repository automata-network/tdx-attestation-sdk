# Automata DCAP Risc Zero Rust Sample Project

Leveraging from the Risc Zero Rust Starter Template, this is the Risc Zero Sample Project to perform the Intel SGX / Intel TDX Quote Verification, including the host and guest program.

## Quick Start

First, make sure [rustup] is installed. The
[`rust-toolchain.toml`][rust-toolchain] file will be used by `cargo` to
automatically install the correct version.

To build all methods and execute the method within the zkVM, run the following
command:

```bash
cargo run
```

### Executing the project locally in development mode

During development, faster iteration upon code changes can be achieved by leveraging [dev-mode], we strongly suggest activating it during your early development phase. Furthermore, you might want to get insights into the execution statistics of your project, and this can be achieved by specifying the environment variable `RUST_LOG="info" RISC0_INFO=1` before running your project.

Put together, the command to run your project in development mode while getting execution statistics is:

```bash
RUST_LOG="info" RISC0_INFO=1 RISC0_DEV_MODE=1 cargo run
```

## Directory Structure

It is possible to organize the files for these components in various ways.
However, in this starter template we use a standard directory structure for zkVM
applications, which we think is a good starting point for your applications.

```text
project_name
├── Cargo.toml
├── host
│   ├── Cargo.toml
│   └── src
│       └── main.rs                        <-- [Host code goes here]
└── methods
    ├── Cargo.toml
    ├── build.rs
    ├── guest
    │   ├── Cargo.toml
    │   └── src
    │       └── bin
    │           └── method_name.rs         <-- [Guest code goes here]
    └── src
        └── lib.rs
```

## Deterministic Build

To build the guest program deterministically and obtain the Image ID consistent with production, you must set the environment variable `RISC0_USE_DOCKER=1` when building.