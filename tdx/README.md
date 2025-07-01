## Intel TDX Quote Generation SDK

## Getting Started
  
### Hardware Requirements
The following cloud service providers (CSP) have support for Intel TDX:

### GCP
- Instance Type: c3-standard-* family
- Operating System: containerOS, RHEL0, SLES-15-sp5, Ubuntu 22.04
- Supported Zones: asia-southeast-1-{a,b,c}, europe-west4-{a,b}, us-central1-{a,b,c} 
- For more information on supported operating systems, please check out the following article on GCP: [supported configurations](https://cloud.google.com/confidential-computing/confidential-vm/docs/supported-configurations#intel-tdx)
- Currently, TDX enabled VMs can only be created via gcloud or Rest API, please check out this article on how to do so: [create an instance](https://cloud.google.com/confidential-computing/confidential-vm/docs/create-a-confidential-vm-instance#gcloud)

#### Azure
- Instance Type: DCesv5-series, DCedsv5-series, ECesv5-series, ECedsv5-series
- Operating System:  Ubuntu 24.04 Server (Confidential VM)- x64 Gen 2 image, Ubuntu 22.04 Server (Confidential VM) - x64 Gen 2 image.
- Supported Region: West Europe, Central US, East US 2, North Europe

#### Others
- If you wish to use a CSP that is not listed above or run your own host, please ensure that the CSP or host is running the following specs:
  - Linux Kernel >= 6.7
  - Virtual Machine (VM) runs under KVM hypervisor 
  - VM has access to `/sys/kernel/config/tsm/report` and able to create a temporary directory with sudo (eg. `sudo mkdir /sys/kernel/config/tsm/report/testing123`).
> If you receive the error `mkdir: cannot create directory ‘testing123’: No such device or address`, it means that ConfigFS is not supported on your VM.

### Download Dependencies
```bash
sudo apt install build-essential pkg-config libtss2-dev
```
### Getting Started with Rust

First, install Rust, and select the default toolchain as nightly.
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```
To get a quick introduction on how to fetch the FMSPC in your TDX machine, we have an example at `examples/fmspc.rs`. To run the example:
```bash
cargo build --example fmspc
sudo ./target/debug/examples/fmspc
```
The example should successfully generate an attestation report on any TDX enabled virtual machine and display the corresponding FMSPC on stdout.

To get a quick introduction on how to generate and verify an attestation report, we have an example at `examples/attestation.rs`. To run the example:
```bash
cargo build --example attestation
sudo ./target/debug/examples/attestation
```
The example should successfully generate and verify an attestation report on any TDX enabled virtual machine and display the result on stdout.

## Rust API Usage

### Initialize Tdx object

In order to run the next few steps, first initialize a Tdx object:

```rust
use tdx::Tdx;

...


let tdx = Tdx::new();
```

### Generate Attestation
To generate an attestation with default options, you can do so like this:
```rust
let (report, _) = tdx.get_attestation_report()?;
```

If you wish to customise options for the attestation report, you can do something like this:

```rust
use tdx::device::DeviceOptions;

...

tdx.get_attestation_report_with_options(
    DeviceOptions {
        report_data: Some([0; 64]),
    }
)?;
```

For details on the struct options, please check out the comments in the struct.

### Verify Attestation
#### Verify Attestation on-chain
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

#### Verify Attestation off-chain
Please follow Intel official DCAP repo [SGXDataCenterAttestationPrimitives](https://github.com/intel/SGXDataCenterAttestationPrimitives) to perform the off-chain verification.
