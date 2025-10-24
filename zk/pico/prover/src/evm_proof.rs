use num::Num;
use num_bigint::BigInt;
use serde_json::json;
use std::{
    fs::{self, File},
    io::{BufReader, Write},
};

const GROTH16_JSON_FILE: &str = "groth16_witness.json";
const PV_FILE: &str = "pv_file";
const PROOF_FILE: &str = "proof.data";
const CONTRACT_INPUTS_FILE: &str = "inputs.json";

pub fn generate_contract_inputs() {
    let pico_out_dir = std::env::current_dir()
        .expect("Failed to get current directory")
        .join("evm_proof_data");

    let proof_path = pico_out_dir.join(PROOF_FILE);
    if !proof_path.exists() {
        panic!("Proof file does not exist at {}", proof_path.display());
    }

    let witness_path = pico_out_dir.join(GROTH16_JSON_FILE);
    if !witness_path.exists() {
        panic!("Witness file does not exist at {}", witness_path.display());
    }

    // Create inputs.json file
    let contract_input_path = pico_out_dir.join(CONTRACT_INPUTS_FILE);
    let mut contract_input_file =
        File::create(&contract_input_path).expect("Failed to create contract input file");

    // Get vkey_hash from witness file
    let witness_file = File::open(&witness_path).expect("Failed to open witness file");
    let witness_reader = BufReader::new(witness_file);
    let witness_json: serde_json::Value =
        serde_json::from_reader(witness_reader).expect("Failed to parse witness file");
    let vkey_hash_str = witness_json["vkey_hash"]
        .as_str()
        .expect("vkey_hash not found in witness file");
    let vkey_hash_bigint =
        BigInt::from_str_radix(vkey_hash_str, 10).expect("Failed to parse vkey hash");
    let vkey_hex_string = format!("{:x}", vkey_hash_bigint);
    let vkey_hex = format!("0x{:0>64}", vkey_hex_string);

    // Get proof from proof.data
    let proof_data = fs::read_to_string(&proof_path).expect("Failed to read proof file");
    let proof_slice: Vec<String> = proof_data.split(',').map(|s| s.to_string()).collect();
    let proof = &proof_slice[0..8];

    // Get pv stream from pv file
    let pv_file_path = pico_out_dir.join(PV_FILE);
    if !pv_file_path.exists() {
        panic!("PV file does not exist at {}", pv_file_path.display());
    }
    let pv_file_content = fs::read_to_string(&pv_file_path).expect("Failed to read pv file");
    let pv_string = pv_file_content.trim();
    if !pv_string[2..].chars().all(|c| c.is_ascii_hexdigit()) {
        panic!("Invalid hex format in pv file");
    }

    let public_values_hex = "0x".to_string() + pv_string;
    let result_json = json!({
        "riscvVKey": vkey_hex,
        "proof": proof,
        "publicValues": public_values_hex
    });

    let json_string = serde_json::to_string_pretty(&result_json).expect("Failed to serialize JSON");
    
    log::info!("Contract input JSON: {}", json_string);
    
    contract_input_file
        .write_all(json_string.as_bytes())
        .expect("Failed to write contract input file");
}
