use dcap_rs::types::{VerifiedOutput, collateral::Collateral};
use pccs_reader_rs::{
    dotenvy, find_missing_collaterals_from_quote, tcb_pem::generate_tcb_issuer_chain_pem,
};
use pico_dcap_core::GuestInput;
use pico_sdk::init_logger;
use prover::{cli, evm_proof::generate_contract_inputs, prove};

use clap::Parser;
use cli::*;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    // Initialize logger
    init_logger();

    match &cli.command {
        Commands::Deserialize(args) => {
            let output_vec =
                hex::decode(remove_prefix_if_found(&args.output)).expect("Failed to parse output");
            let deserialized_output = VerifiedOutput::from_bytes(&output_vec);
            println!("Deserialized output: {:?}", deserialized_output);
        }
        Commands::Parse(args) => {
            let input_bytes = std::fs::read(&args.input).expect("Failed to read input file");
            let guest_input = GuestInput::sol_abi_decode(&input_bytes);
            println!("Parsed Guest Input: {:?}", guest_input);
        }
        Commands::GenerateEvmInputs => {
            generate_contract_inputs();
        }
        Commands::Prove(args) => {
            let input_bytes = if let Some(input_path) = &args.guest_input_path {
                std::fs::read(input_path).expect("Failed to read guest input file")
            } else {
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
                let encoded_input = guest_input.sol_abi_encode();
                std::fs::write("input.bin", &encoded_input).expect("Failed to write input.bin");
                encoded_input
            };

            log::debug!("Guest input: {}", hex::encode(&input_bytes));
            prove(&input_bytes);
        }
    }
}
