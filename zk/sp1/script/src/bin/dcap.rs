use dcap_script::cli::*;

use clap::Parser;
use dcap_rs::types::collateral::Collateral;
use dcap_rs::types::VerifiedOutput;
use pccs_reader_rs::{
    dotenvy, find_missing_collaterals_from_quote, tcb_pem::generate_tcb_issuer_chain_pem,
};
use core::GuestInput;
use sp1_sdk::{utils, HashableKey, ProverClient, SP1Stdin, include_elf, network::FulfillmentStrategy, Prover};

pub const DCAP_ELF: &[u8] = include_elf!("dcap-sp1-guest-program");

#[tokio::main]
async fn main() {
    utils::setup_logger();

    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Deserialize(args) => {
            let output_vec =
                hex::decode(remove_prefix_if_found(&args.output)).expect("Failed to parse output");
            let deserialized_output = VerifiedOutput::from_bytes(&output_vec);
            println!("Deserialized output: {:?}", deserialized_output);
        },
        Commands::Prove(args) => {
            let raw_quote = get_quote(&args.quote_path, &args.quote_hex);

            let fetched_collaterals =
                find_missing_collaterals_from_quote(raw_quote.as_slice(), false)
                    .await
                    .unwrap();

            log::debug!("Fetched collaterals: {:?}", fetched_collaterals);

            let tcb_issuer_chain_pem = generate_tcb_issuer_chain_pem(
                fetched_collaterals.tcb_signing_ca.as_slice(),
                fetched_collaterals.root_ca.as_slice(),
            );

            let collateral = Collateral::new(
                fetched_collaterals.root_ca_crl.as_slice(),
                fetched_collaterals.pck_crl.as_slice(),
                tcb_issuer_chain_pem.unwrap().as_bytes(),
                fetched_collaterals.tcb_info.as_str(),
                fetched_collaterals.qe_identity.as_str(),
            )
            .unwrap();

            // get current time in seconds since epoch
            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let guest_input = GuestInput {
                raw_quote: raw_quote.to_vec(),
                collateral: collateral,
                timestamp: current_time,
            };

            let input_bytes = guest_input.sol_abi_encode();

            let mut stdin = SP1Stdin::new();
            stdin.write_slice(&input_bytes);

            let network_mode = args.network_prover_mode.unwrap();
            let client = ProverClient::builder().network().build();
            let strategy = match network_mode {
                NetworkProverMode::Auction => FulfillmentStrategy::Auction,
                NetworkProverMode::Hosted => FulfillmentStrategy::Hosted,
                NetworkProverMode::Reserved => FulfillmentStrategy::Reserved
            };

            // Execute the program first
            let (ret, report) = client.execute(DCAP_ELF, &stdin).run().unwrap();
            log::info!(
                "executed program with {} cycles",
                report.total_instruction_count()
            );

            // Generate the proof
            let (pk, vk) = client.setup(DCAP_ELF);
            let proof_system = args.proof_system.unwrap();
            let proof = match proof_system {
                ProofSystem::Groth16 => client.prove(&pk, &stdin).groth16().strategy(strategy).run().unwrap(),
                ProofSystem::Plonk => client.prove(&pk, &stdin).plonk().strategy(strategy).run().unwrap()
            };

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
            println!("VK (onchain): {}", vk.bytes32().to_string().as_str());
            println!("VK hash bytes: {}", hex::encode(vk.hash_bytes()));
            println!("Proof: {}", hex::encode(&proof_bytes));
            println!("Proof selector: {}", hex::encode(&proof_bytes[..4]));

            let parsed_output = VerifiedOutput::from_bytes(&output);
            println!("{:?}", parsed_output);

            // Verify proof
            client.verify(&proof, &vk).expect("Failed to verify proof");
            println!("Successfully verified proof.");
        }
    }
}
