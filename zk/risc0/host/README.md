<div align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/automata-network/automata-brand-kit/main/PNG/ATA_White%20Text%20with%20Color%20Logo.png">
    <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/automata-network/automata-brand-kit/main/PNG/ATA_Black%20Text%20with%20Color%20Logo.png">
    <img src="https://raw.githubusercontent.com/automata-network/automata-brand-kit/main/PNG/ATA_White%20Text%20with%20Color%20Logo.png" width="50%">
  </picture>
</div>

# Automata DCAP with Bonsai CLI Guide
[![Automata DCAP Bonsai CLI](https://img.shields.io/badge/Power%20By-Automata-orange.svg)](https://github.com/automata-network)

## Summary

This CLI tool is used to fetch SNARK proofs of execution on the DCAP Guest Application via Bonsai, and optionally submit them on-chain. The DCAP Guest Application proves that an Intel SGX DCAP quote has been successfully verified and the enclave which originated the quote is legitimate.

Before you begin, make sure to `cd` into this directory.

Follow these steps to get started with this tool:

0. Install [Rust](https://doc.rust-lang.org/book/ch01-01-installation.html)

1. Export `BONSAI_API_KEY` and `BONSAI_API_URL` values into the shell. If you don't have a Bonsai API key, send a [request](https://docs.google.com/forms/d/e/1FAIpQLSf9mu18V65862GS4PLYd7tFTEKrl90J5GTyzw_d14ASxrruFQ/viewform) for one.

```bash
export BONSAI_API_KEY="" # see form linked above
export BONSAI_API_URL="" # provided with your api key
```

Alternatively, you may simply source `.env` into shell.

2. Build the program.

```bash
cargo build --release
```

---

## CLI Commands

You may run the following command to see available commands.

```bash
./target/release/prover --help
```

Outputs:

```bash
Gets Bonsai Proof for DCAP Quote Verification and submits on-chain

Usage: prover <COMMAND>

Commands:
  prove        Fetches proof from Bonsai and sends them on-chain to verify DCAP quote
  image-id     Computes the Image ID of the Guest application
  deserialize  De-serializes and prints information about the Output
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

```

To get help on individual commands (e.g. `prove`), do the following:

```bash
./target/release/prover prove --help
```

Output:

```bash
Fetches proof from Bonsai and sends them on-chain to verify DCAP quote

Usage: prover prove [OPTIONS]

Options:
  -q, --quote-hex <QUOTE_HEX>
          The input quote provided as a hex string, this overwrites the --quote-path argument
  -p, --quote-path <QUOTE_PATH>
          Optional: The path to a quote.hex file. Default: /data/quote.hex or overwritten by the --quote-hex argument if provided
  -k, --wallet-key <WALLET_PRIVATE_KEY>
          Optional: A transaction will not be sent if left blank
  -h, --help
          Print help
```

---

## Get Started

You may either pass your quote as a hexstring with the `--quote-hex` flag, or as a stored hexfile in `/data/quote.hex`. If you store your quote elsewhere, you may pass the path with the `--quote-path` flag.

>
> [!NOTE]
> Beware that passing quotes with the `--quote-hex` flag overwrites passing quotes with the `--quote-path` flag.
>

It is also recommended to set the environment value `RUST_LOG=info` to view logs.

To begin, run the command below:

```bash
RUST_LOG=info ../target/release/prover prove
```
>
> [!NOTE]
> Passing your wallet key is optional. If none is provided, the program verifies the journal and seal with a staticcall made to the contract, without sending a transaction.
>

>
> [!NOTE]
> You may run the command below to check the computed ImageID for the provided Guest program ELF.
>
> ``` bash
> ./target/release/prover image-id
> ```
>