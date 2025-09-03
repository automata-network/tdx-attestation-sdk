# Automata DCAP SP1 Project

Leveraging from the [SP1](https://github.com/succinctlabs/sp1) project template, we introduce the Automata DCAP SP1 project to perform the Intel SGX / Intel TDX Quote Verification, including the host and guest program.

## Requirements

- [Rust](https://rustup.rs/)
- [SP1](https://docs.succinct.xyz/getting-started/install.html)
- [Docker](https://docs.docker.com/get-started/get-docker/)

## Running the Project

There are four main steps to run this project: build a program, execute a program, generate a core proof, and generate an EVM-compatible proof.

### Build the Program

To build the program, run the following command:

```sh
cd program
cargo prove build
```

### Execute the Program and Generate the Proof

> [!WARNING]
> You will need at least 128 GB of RAM to generate a Groth16 or PLONK proof if you want to perform the calculation locally.

To generate a proof that is small enough to be verified on-chain and verifiable by the EVM:

```sh
cd script
cargo run --release
```

This will generate a Groth16 proof. If you want to generate a PLONK proof, use the `plonk()` method as shown below.

```rust
let proof = client.prove(&pk, stdin.clone()).plonk().run().unwrap();
```

These commands will also generate fixtures that can be used to test the verification of SP1 zkVM proofs
inside Solidity.

Pay attention, if you want to use [Automata DCAP Attestation](https://github.com/automata-network/automata-dcap-attestation) to perform the on-chain verification, we recommend using [dcap-sp1-cli](/clis/dcap-sp1-cli) to generate the zkVM proofs with the same Verification Key.

### Retrieve the Verification Key

To retrieve your `programVKey` for your on-chain contract, run the following command:

```sh
cargo prove vkey --elf elf/dcap-sp1-guest-program-elf
```

## Using the Prover Network

We highly recommend using the Succinct prover network for any non-trivial programs or benchmarking purposes. For more information, see [quickstart](hhttps://docs.succinct.xyz/docs/sp1/prover-network/quickstart).

To get started, copy the example environment file:

```sh
cp .env.example .env
```

Then, set the `NETWORK_PRIVATE_KEY` environment variable to your whitelisted private key.

For example, to generate an EVM-compatible proof using the prover network, run the following
command:

```sh
NETWORK_PRIVATE_KEY=... cargo run --release
```

### Prover Network Strategies

The CLI currently supports the following network proving strategies:

- **Hosted Strategy:**

This is the strategy selected at default, which uses Succinct on-demand prover network.

- **Reserved Strategy:**

Uses network with reserved capacity upon agreement with Succinct. Read [reserved capacity](https://docs.succinct.xyz/docs/sp1/prover-network/reserved-capacity) from the docs to learn more.

- **Auction Strategy:**

Requests proofs directly on the mainnet using an auctioning mechanism. You must make sure that your account is funded with $PROVE tokens.

> [!NOTE]
>
> You must disable `reserved-capacity` feature in the `sp1-sdk` crate to use the Auction strategy.