use dcap_rs::types::collateral::Collateral;
use core::GuestInput;
use sp1_sdk::{utils, HashableKey, ProverClient, SP1Stdin, include_elf};

pub const DCAP_ELF: &[u8] = include_elf!("dcap-sp1-guest-program");

fn main() {
    utils::setup_logger();

    let raw_quote = include_bytes!("../../data/quote.dat");

    let collateral = Collateral::new(
        include_bytes!("../../data/intel_root_crl.der"),
        include_bytes!("../../data/pck_platform_crl.der"),
        include_bytes!("../../data/signing_cert.pem"),
        include_str!("../../data/tcbinfo-tdx-v3.json"),
        include_str!("../../data/identity_tdx.json"),
    )
    .unwrap();

    // // get current time in seconds since epoch
    // let current_time = std::time::SystemTime::now()
    //     .duration_since(std::time::UNIX_EPOCH)
    //     .unwrap()
    //     .as_secs();

    let current_time = 1749095100u64;

    let guest_input = GuestInput {
        raw_quote: raw_quote.to_vec(),
        collateral: collateral,
        timestamp: current_time,
    };

    let input_string = serde_json::to_string(&guest_input).unwrap();

    let mut stdin = SP1Stdin::new();
    stdin.write(&input_string);

    let client = ProverClient::from_env();

    // Execute the program first
    let (ret, report) = client.execute(DCAP_ELF, &stdin).run().unwrap();
    println!(
        "executed program with {} cycles",
        report.total_instruction_count()
    );
    // println!("{:?}", report);

    // Generate the proof
    let (pk, vk) = client.setup(DCAP_ELF);
    let proof = client.prove(&pk, &stdin).groth16().run().unwrap();
    // let proof = client.prove(&pk, &stdin).plonk().run().unwrap();

    let ret_slice = ret.as_slice();
    let output_len = u16::from_be_bytes([ret_slice[0], ret_slice[1]]) as usize;
    let mut output = Vec::with_capacity(output_len);
    output.extend_from_slice(&ret_slice[2..2 + output_len]);

    let proof_bytes = proof.bytes();

    println!("Execution Output: {}", hex::encode(ret_slice));
    println!(
        "Proof pub value: {}",
        hex::encode(proof.public_values.as_slice())
    );
    println!("VK: {}", vk.bytes32().to_string().as_str());
    println!("Proof: {}", hex::encode(&proof_bytes));
    println!("Proof selector: {}", hex::encode(&proof_bytes[..4]));

    // let parsed_output = VerifiedOutput::from_bytes(&output);
    // println!("{:?}", parsed_output);

    // Verify proof
    client.verify(&proof, &vk).expect("Failed to verify proof");
    println!("Successfully verified proof.");
}
