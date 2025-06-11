<div align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/automata-network/automata-brand-kit/main/PNG/ATA_White%20Text%20with%20Color%20Logo.png">
    <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/automata-network/automata-brand-kit/main/PNG/ATA_Black%20Text%20with%20Color%20Logo.png">
    <img src="https://raw.githubusercontent.com/automata-network/automata-brand-kit/main/PNG/ATA_White%20Text%20with%20Color%20Logo.png" width="50%">
  </picture>
</div>

# Automata TDX Attestation SDK
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)

## Overview

Automata TDX Attestation SDK is the most-feature complete SDK for Intel TDX development, it consists of two parts:

* TDX package: it helps developers to generate the Intel TDX Quote in different cloud service providers (CSP).
* Risc0 and Succinct ZK host and guest programs.

### Environment Preparation
Refer to [TDX package](tdx/README.md) to setup the Intel TDX CVM in different cloud service providers (CSP).

## Intel TDX Quote Generation
Use [TDX package](tdx/README.md) to generate the Intel TDX Quote, you can find an example in [tdx_attestation](tdx/examples/attestation.rs).

## Intel TDX Quote Verification
### Verify Attestation on-chain
In [Automata DCAP Attestation](https://github.com/automata-network/automata-dcap-attestation), We provide two ways to verify the Intel TDX quote on-chain:

```solidity
function verifyAndAttestOnChain(bytes calldata rawQuote)
```
It accepts the raw quote hex string to perform the on-chain verification, all collaterals will be fetched from the [Automata on-chain PCCS](https://github.com/automata-network/automata-on-chain-pccs).

```solidity
function verifyAndAttestWithZKProof(bytes calldata output, ZkCoProcessorType zkCoprocessor, bytes calldata proofBytes)
```
The first parameter represents the output of the zkVM, the second one is the zkVM type, and the third one is its corresponding proof. It supports two kinds of ZK technologies to perform the on-chain verification:

* [Risc0](https://github.com/risc0/risc0)
  - output: the journal of the Risc0 zkVM output
  - zkCoprocessor: 1
  - proofBytes: the seal of the Risc0 zkVM output

* [SP1](https://github.com/succinctlabs/sp1)
  - output: the execution result of the SP1 Prover output
  - zkCoprocessor: 2
  - proofBytes: the proof of the SP1 Prover output

The on-chain verification contract has been deployed to Automata Testnet at [0x95175096a9B74165BE0ac84260cc14Fc1c0EF5FF](https://explorer-testnet.ata.network/address/0x95175096a9B74165BE0ac84260cc14Fc1c0EF5FF).

The [ImageID](https://dev.risczero.com/terminology#image-id) currently used for the DCAP RiscZero Guest Program is `4cf071b3cc25d73e77f430b65f5700dd53522dacc21c1bfc0862b2e46fda3584`.

The [VKEY](https://docs.succinct.xyz/verification/onchain/solidity-sdk.html?#finding-your-program-vkey) currently used for the DCAP SP1 Program is
`0036efd519bb371b29a40322e40031833716e9441c6907f8aefc5e52ceebc9a6`.

An useful DCAP zkVM clis can be found at [Automata DCAP zkVM CLI](https://github.com/automata-network/automata-dcap-zkvm-cli).

### Verify Attestation off-chain
Please follow the Intel official DCAP repo [SGXDataCenterAttestationPrimitives](https://github.com/intel/SGXDataCenterAttestationPrimitives) to perform the off-chain verification.

## ZK Optimization
### Risc0
To get started, you need to have the following installed:

* [Rust](https://doc.rust-lang.org/cargo/getting-started/installation.html)
* [Foundry](https://getfoundry.sh/)
* [RISC Zero](https://dev.risczero.com/api/zkvm/install)

#### Configuring Bonsai

***Note:*** *To request an API key [complete the form here](https://bonsai.xyz/apply).*

With the Bonsai proving service, you can produce a [Groth16 SNARK proof] that is verifiable on-chain.
You can get started by setting the following environment variables with your API key and associated URL.

```bash
export BONSAI_API_KEY="YOUR_API_KEY" # see form linked above
export BONSAI_API_URL="BONSAI_URL" # provided with your api key
```

### Succinct
To get started, you need to have the following installed:

* [Rust](https://doc.rust-lang.org/cargo/getting-started/installation.html)
* [SP1](https://docs.succinct.xyz/getting-started/install.html)
* [Docker](https://docs.docker.com/get-started/get-docker/)

***Note:*** *To request an whitelisted address, [complete the form here](https://docs.google.com/forms/d/e/1FAIpQLSd-X9uH7G0bvXH_kjptnQtNil8L4dumrVPpFE4t8Ci1XT1GaQ/viewform).*

With the SP1 Proving Network, you can produce a [Groth16 SNARK proof] or [Plonk SNARK proof] that is verifiable on-chain.
You can get started by setting the following environment variables with your whitelisted address and associated Proving Network.

```bash
export SP1_PROVER=network
export SP1_PRIVATE_KEY=""
```

## Acknowledgements
We would like to acknowledge the projects below whose previous work has been instrumental in making this project a reality.

* [Risc0](https://github.com/risc0/risc0): The Risc0 ZK Optimization to reduce the gas cost to verify the Intel TDX Quote on-chain.
* [SP1](https://github.com/succinctlabs/sp1): The Succinct ZK Optimization to reduce the gas cost to verify the Intel TDX Quote on-chain. It supports Groth16 and Plonk proofs.

## Disclaimer
This project is under development. All source code and features are not production ready.
