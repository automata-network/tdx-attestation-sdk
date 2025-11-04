use methods::{DCAP_GUEST_ELF, DCAP_GUEST_ID};
use risc0_ethereum_contracts::groth16;
use risc0_zkvm::{default_prover, ExecutorEnv, InnerReceipt, ProverOpts};
use dcap_rs::types::VerifiedOutput;

use crate::cli::DcapArgs;

pub async fn prove_with_bonsai(input_bytes: &[u8]) {
    let env = ExecutorEnv::builder()
        .write_slice(&input_bytes)
        .build()
        .unwrap();

    // Obtain the default prover.
    let prover = default_prover();
    // Produce a receipt by proving the specified ELF binary.
    let prover_opts = if std::env::var("BONSAI_API_KEY").is_ok() {
        ProverOpts::groth16()
    } else {
        ProverOpts::default()
    };
    let receipt = prover
        .prove_with_opts(env, DCAP_GUEST_ELF, &prover_opts)
        .unwrap()
        .receipt;

    if let InnerReceipt::Groth16(ref groth16_receipt) = receipt.inner {
        println!(
            "Groth16 Seal bytes: {}",
            hex::encode(
                groth16::encode(groth16_receipt.seal.clone())
                    .unwrap()
                    .as_slice()
            )
        );
        println!(
            "Output bytes: {}",
            hex::encode(receipt.journal.bytes.clone().as_slice())
        )
    }

    // verify your receipt
    receipt.verify(DCAP_GUEST_ID).unwrap();

    let output = receipt.journal.bytes;

    // manually parse the output
    let mut offset: usize = 0;
    let output_len = u16::from_be_bytes(output[offset..offset + 2].try_into().unwrap());

    offset += 2;
    let verified_output =
        VerifiedOutput::from_bytes(&output[offset..offset + output_len as usize]).unwrap();
    offset += output_len as usize;
    let current_time = u64::from_be_bytes(output[offset..offset + 8].try_into().unwrap());
    offset += 8;
    let tcbinfo_root_hash = &output[offset..offset + 32];
    offset += 32;
    let enclaveidentity_root_hash = &output[offset..offset + 32];
    offset += 32;
    let root_cert_hash = &output[offset..offset + 32];
    offset += 32;
    let signing_cert_hash = &output[offset..offset + 32];
    offset += 32;
    let root_crl_hash = &output[offset..offset + 32];
    offset += 32;
    let pck_crl_hash = &output[offset..offset + 32];

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

pub async fn prove_with_boundless(input_bytes: &[u8], args: &DcapArgs) {
    use boundless_market::{
        alloy::{
            primitives::U256,
            providers::{Provider, ProviderBuilder},
            signers::local::PrivateKeySigner,
            transports::http::reqwest::Url,
        },
        Deployment,
    };
    use crate::boundless::{request_proof, ProofRequestConfig};

    // Validate required args for Boundless
    let rpc_url = args
        .boundless_rpc_url
        .as_ref()
        .expect("--boundless-rpc-url or BOUNDLESS_RPC_URL is required for boundless strategy");
    let private_key_hex = args
        .boundless_private_key
        .as_ref()
        .expect("--boundless-private-key or ETH_WALLET_PRIVATE_KEY is required for boundless strategy");

    // Parse RPC URL
    let rpc_url_parsed: Url = rpc_url
        .parse()
        .expect("Invalid RPC URL format");

    // Get chain deployment
    let provider = ProviderBuilder::new().connect_http(rpc_url_parsed.clone());
    let chain_id = provider
        .get_chain_id()
        .await
        .expect("Failed to retrieve chain ID");
    log::info!("Detected chain ID: {}", chain_id);

    let deployment = Deployment::from_chain_id(chain_id)
        .expect(&format!("No Boundless deployment found for chain ID {}", chain_id));

    // Parse private key
    let private_key_bytes = hex::decode(private_key_hex)
        .expect("Failed to decode private key");
    let private_key = PrivateKeySigner::from_slice(&private_key_bytes)
        .expect("Failed to create signer from private key");

    // Build config for proof request
    let config = ProofRequestConfig {
        rpc_url: rpc_url_parsed,
        private_key,
        deployment,
        program_url: args.boundless_program_url.clone(),
        proof_type: args.boundless_proof_type,
        min_price: args.boundless_min_price.map(U256::from),
        max_price: args.boundless_max_price.map(U256::from),
        timeout: args.boundless_timeout,
        ramp_up_period: args.boundless_ramp_up_period,
    };

    // Request proof from Boundless
    log::info!("Requesting proof from Boundless network...");
    let (journal, seal) = request_proof(input_bytes, DCAP_GUEST_ELF, config)
        .await
        .expect("Failed to request proof from Boundless");

    println!("Journal: {}", journal.to_string());
    println!("Seal: {}", seal.to_string());

    // Optionally verify on-chain (similar to reference implementation)
    // For now, just output the results
    log::info!("Proof request completed successfully");
}
