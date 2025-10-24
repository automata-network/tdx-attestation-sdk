<div align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/automata-network/automata-brand-kit/main/PNG/ATA_White%20Text%20with%20Color%20Logo.png">
    <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/automata-network/automata-brand-kit/main/PNG/ATA_Black%20Text%20with%20Color%20Logo.png">
    <img src="https://raw.githubusercontent.com/automata-network/automata-brand-kit/main/PNG/ATA_White%20Text%20with%20Color%20Logo.png" width="50%">
  </picture>
</div>

# Automata DCAP Pico zkVM Project

Leveraging the [Pico zkVM](https://github.com/brevis-network/pico) framework, this project performs Intel SGX / Intel TDX Quote Verification with zero-knowledge proofs that can be verified on-chain.

## Requirements

- [Rust](https://rustup.rs/) - The [`rust-toolchain`](./rust-toolchain) file specifies the required nightly version
- [cargo-pico](https://docs.brevis.network/getting-started/quick-start) - Pico zkVM build tools
- Minimum memory requirement: [32GB for EVM proof generation](https://docs.brevis.network/getting-started/quick-start#start-with-the-evm-template)

> [NOTE]
>
> We advise you use a machine that is equipped with at least 256GB of memory to run the 
> prover.
>
> It took us about an hour to generate EVM proofs (over Koalabear Field) by running the zkVM
> on an [Azure NC40ads H100 v5](https://learn.microsoft.com/en-us/azure/virtual-machines/sizes/gpu-accelerated/ncadsh100v5-series?tabs=sizebasic) instance.
>
> Currently the Pico SDK does not support GPU proving, and we believe the proving speed
> will significantly improve after GPU support is enabled.

## Quick Start

### 1. Install cargo-pico

Follow the [Pico installation guide](https://docs.brevis.network/getting-started/installation) to install the cargo-pico CLI tool.

### 2. Set up environment variables

Copy the example environment file and configure it with your settings:

```bash
cp .env.example .env
```

Edit `.env` to set the required values:
- `RPC_URL`: RPC endpoint for the target network
- `ENCLAVE_ID_DAO`: Address of the Enclave ID DAO contract
- `FMSPC_TCB_DAO`: Address of the FMSPC TCB DAO contract
- `PCS_DAO`: Address of the PCS DAO contract

### 3. Build the guest program

The guest program runs inside the zkVM and performs the DCAP quote verification:

```bash
cd app
cargo pico build
```

This generates the RISC-V ELF binary at `app/elf/riscv32im-pico-zkvm-elf`.

### 4. Build the host program

The host program (prover) prepares inputs and generates proofs:

```bash
cd prover
cargo build --release
```

### 5. Generate a proof

To verify a DCAP quote and generate an EVM-compatible proof:

```bash
cd prover
cargo run --release -- prove --quote-path ./data/quote.hex
```

Or provide a quote directly as a hex string:

```bash
cargo run --release -- prove --quote-hex "0x..."
```

## Directory Structure

```text
pico/
├── app
│   ├── Cargo.toml
│   ├── elf
│   │   └── riscv32im-pico-zkvm-elf
│   └── src
│       └── main.rs
├── core
│   ├── Cargo.toml
│   └── src
│       └── lib.rs
├── prover
│   ├── Cargo.toml
│   ├── data
│   │   ├── quote.dat
│   │   └── quote.hex
│   ├── evm_proof_data
│   │   ├── Groth16Verifier.sol
│   │   ├── inputs.json
│   │   ├── vm_pk
│   │   └── vm_vk
│   ├── input.bin
│   └── src
│       ├── cli.rs
│       ├── evm_proof.rs
│       ├── lib.rs
│       └── main.rs
└─
```

## Prover Commands

The prover supports several commands:

### Generate Proof

Verify a DCAP quote and generate a proof:

```bash
cargo run --release -- prove [OPTIONS]
```

**Options:**
- `--quote-path <PATH>`: Path to a quote.hex file (default: `./data/quote.hex`)
- `--quote-hex <HEX>`: Quote provided as hex string (overrides `--quote-path`)
- `--input <PATH>`: Use pre-generated guest input (skips collateral fetching)

The prover will:
1. Fetch required collaterals from [Automata Onchain PCCS](https://github.com/automata-network/automata-on-chain-pccs) (if not using pre-generated input)
2. Encode the guest input
3. Generate an EVM-compatible proof
4. Output proof artifacts to `evm_proof_data/`

### Parse Input

Parse and inspect a generated input.bin file:

```bash
cargo run --release -- parse --input-path input.bin [--output-dir <DIR>]
```

Use `--output-dir` to extract collaterals to a directory.

### Deserialize Output

Deserialize and display a verified output:

```bash
cargo run --release -- deserialize --output <HEX>
```

### Generate EVM Contract Inputs

Generate formatted inputs from proof artifacts for on-chain verification:

```bash
cargo run --release -- generate-evm-inputs
```

## Proof Generation

For on-chain verification, the first run will perform a trusted setup and save the proving key to `evm_proof_data/vm_pk`. Subsequent runs will reuse this key.

**Generated artifacts:**
- `vm_pk`: Proving key (generated on first run)
- `proof_file`: The zero-knowledge proof
- `pv_file`: Public values (output from the zkVM)
- `vk_file`: Verification key
- `input.json`: The formatted inputs for onchain verification
- `Groth16Verifier.sol`: The Groth16 Verifier contract that corresponds to the proving key

### Advanced Configuration

To optimize the proving process for your system's requirement, we advise tweaking the `CHUNK_SIZE` and `CHUNK_BATCH_SIZE` environmental values. 

## Development Tips

- Use `RUST_LOG=debug` to see detailed logging output
- The `input.bin` file is saved after fetching collaterals and can be reused with `--input`
- The first EVM proof generation takes longer due to trusted setup

## Related Projects

- [dcap-rs](https://github.com/automata-network/dcap-rs) - Intel DCAP attestation verification library
- [Automata DCAP Attestation](https://github.com/automata-network/automata-dcap-attestation) - On-chain verification contracts
- [Pico zkVM](https://github.com/brevis-network/pico) - Zero-knowledge virtual machine
