<div align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/automata-network/automata-brand-kit/main/PNG/ATA_White%20Text%20with%20Color%20Logo.png">
    <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/automata-network/automata-brand-kit/main/PNG/ATA_Black%20Text%20with%20Color%20Logo.png">
    <img src="https://raw.githubusercontent.com/automata-network/automata-brand-kit/main/PNG/ATA_White%20Text%20with%20Color%20Logo.png" width="50%">
  </picture>
</div>

# Automata DCAP with SP1 CLI Guide
[![Automata DCAP SP1 CLI](https://img.shields.io/badge/Power%20By-Automata-orange.svg)](https://github.com/automata-network)

## Summary

This CLI tool is used to fetch SP1 proofs of execution on the DCAP Guest Application via Succinct Proving Network, and optionally submit them on-chain. The DCAP Guest Application proves that an Intel SGX / Intel TDX DCAP quote has been successfully verified and the enclave which originated the quote is legitimate.

Follow these steps to get started with this tool:

0. To get started, you need to have the following installed:

* [Rust](https://doc.rust-lang.org/cargo/getting-started/installation.html)
* [SP1](https://docs.succinct.xyz/getting-started/install.html)
* [Docker](https://docs.docker.com/get-started/get-docker/)

1. Export `SP1_PROVER` and `NETWORK_PRIVATE_KEY` values into the shell. If you don't have a whitelisted private key, send a [request](https://forms.gle/rTUvhstS8PFfv9B3A) for one.

```bash
export SP1_PROVER=network
export NETWORK_PRIVATE_KEY=""
```

Or alternatively, you may simply create a `.env` file, containing the variables and source it to shell.

2. Build the program.

```bash
cargo build --release
```

---

## CLI Commands

You may run the following command to see available commands.

```bash
../target/release/dcap-sp1-cli --help
```

Outputs:

```bash
Gets SP1 Proof for DCAP Quote Verification and submits on-chain

Usage: dcap-sp1-cli <COMMAND>

Commands:
  prove        Fetches proof from SP1 and sends them on-chain to verify DCAP quote
  deserialize  De-serializes and prints information about the Output
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

```

To get help on individual commands (e.g. `prove`), do the following:

```bash
../target/release/dcap-sp1-cli prove --help
```

Output:

```bash
Fetches proof from SP1 and sends them on-chain to verify DCAP quote

Usage: dcap-sp1-cli prove [OPTIONS]

Options:
  -q, --quote-hex <QUOTE_HEX>        The input quote provided as a hex string, this overwrites the --quote-path argument
  -p, --quote-path <QUOTE_PATH>      Optional: The path to a quote.hex file. Default: /data/quote.hex or overwritten by the --quote-hex argument if provided
  -s, --prove-system <PROOF_SYSTEM>  [default: groth16] [possible values: groth16, plonk]
  -h, --help                         Print help
```

---

## Get Started

You may either pass your quote as a hex string with the `--quote-hex` flag, or as a stored hex file in `/data/quote.hex`. If you store your quote elsewhere, you may pass the path with the `--quote-path` flag.

>
> ℹ️ **Note**
> Beware that passing quotes with the `--quote-hex` flag overwrites passing quotes with the `--quote-path` flag.
>

It is also recommended to set the environment value `RUST_LOG=info` to view logs.

To begin, run the command below:

```bash
RUST_LOG=info ../target/release/dcap-sp1-cli prove
```

>
> ℹ️ **Note**
> The expected VKEY to perform SNARK verification on-chain should be: 
> `004b1c0509dc0eee05ad437e3600027bd8cb3fbb03b29bf9cd839ae2e684c749`
>