pub mod cli;
pub mod evm_proof;

use dcap_rs::types::VerifiedOutput;
use pico_dcap_core::load_elf;
use pico_sdk::client::DefaultProverClient;

pub fn prove(input_bytes: &[u8]) {
    // Load the ELF file
    let elf = load_elf("../app/elf/riscv32im-pico-zkvm-elf");

    // Initialize the prover client
    let client = DefaultProverClient::new(&elf);
    // Initialize new stdin
    let mut stdin_builder = client.new_stdin_builder();

    // Set up input
    stdin_builder.write_slice(&input_bytes);

    // Set up output path
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let output_path = current_dir.join("./evm_proof_data");

    let (cycles, public_buffer) = client.emulate(stdin_builder.clone());
    log::info!("EVM Emulation Cycles: {}", cycles);

    if std::env::var("DEV_MODE").is_err() || std::env::var("DEV_MODE").unwrap() == "" {
        // Generate proof
        let proving_key_path = output_path.join("vm_pk");
        let need_setup = !proving_key_path.exists();
        log::info!("EVM Proving Requires Trusted Setup: {}", need_setup);
        client
            .prove_evm(stdin_builder, need_setup, output_path.clone(), "kb")
            .expect("Failed to generate proof");

        log::info!("Proof generated successfully");
    }

    // manually parse the output
    let mut offset: usize = 0;
    let output_len = u16::from_be_bytes(public_buffer[offset..offset + 2].try_into().unwrap());

    offset += 2;
    let verified_output =
        VerifiedOutput::from_bytes(&public_buffer[offset..offset + output_len as usize]).unwrap();
    offset += output_len as usize;
    let current_time = u64::from_be_bytes(public_buffer[offset..offset + 8].try_into().unwrap());
    offset += 8;
    let tcbinfo_root_hash = &public_buffer[offset..offset + 32];
    offset += 32;
    let enclaveidentity_root_hash = &public_buffer[offset..offset + 32];
    offset += 32;
    let root_cert_hash = &public_buffer[offset..offset + 32];
    offset += 32;
    let signing_cert_hash = &public_buffer[offset..offset + 32];
    offset += 32;
    let root_crl_hash = &public_buffer[offset..offset + 32];
    offset += 32;
    let pck_crl_hash = &public_buffer[offset..offset + 32];

    println!("Verified Output: {:?}", verified_output);
    println!("Current time: {}", current_time);
    println!("TCB Info Root Hash: {:?}", tcbinfo_root_hash);
    println!(
        "Enclave Identity Root Hash: {:?}",
        enclaveidentity_root_hash
    );
    println!("Root Cert Hash: {:?}", root_cert_hash);
    println!("Signing Cert Hash: {:?}", signing_cert_hash);
    println!("RootCRL Hash: {:?}", root_crl_hash);
    println!("PCK CRL Hash: {:?}", pck_crl_hash);
}
