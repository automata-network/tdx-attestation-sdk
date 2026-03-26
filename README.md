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

Automata TDX Attestation SDK is a feature-complete SDK for Intel TDX development. It helps developers to generate the Intel TDX Quote in different cloud service providers (CSP).

### Environment Preparation
Refer to the [TDX package](tdx/README.md) to set up the Intel TDX CVM in different cloud service providers (CSP).

## Intel TDX Quote Generation
Use [TDX package](tdx/README.md) to generate the Intel TDX Quote, you can find an example in [tdx_attestation](tdx/examples/attestation.rs).

## Intel TDX Quote Verification
### Verify Attestation on-chain
In [Automata DCAP Attestation](https://github.com/automata-network/automata-dcap-attestation), we provide the following way to verify the Intel TDX quote on-chain:

```solidity
function verifyAndAttestOnChain(bytes calldata rawQuote)
```
It accepts the raw quote hex string to perform the on-chain verification, all collaterals will be fetched from the [Automata on-chain PCCS](https://github.com/automata-network/automata-on-chain-pccs).

The on-chain verification contract has been deployed to Automata Testnet at [0x95175096a9B74165BE0ac84260cc14Fc1c0EF5FF](https://explorer-testnet.ata.network/address/0x95175096a9B74165BE0ac84260cc14Fc1c0EF5FF).

> **Note:** For ZK proof-based DCAP verification (Risc0 / SP1), please refer to [Automata DCAP Attestation](https://github.com/automata-network/automata-dcap-attestation/tree/staging).

### Verify Attestation off-chain
Please follow the Intel official DCAP repo [SGXDataCenterAttestationPrimitives](https://github.com/intel/SGXDataCenterAttestationPrimitives) to perform the off-chain verification.

## Disclaimer
This project is under development. All source code and features are not production-ready.
