use boundless::attestation::IAttestation;
use boundless_market::{
    alloy::{
        primitives::{address, utils::parse_ether},
        signers::local::PrivateKeySigner,
    },
    client::ClientBuilder,
    contracts::{Input, Offer, Predicate, ProofRequestBuilder, Requirements},
    input::InputBuilder,
    storage::{StorageProviderConfig, StorageProviderType},
};
// use methods::{DCAP_GUEST_ELF, DCAP_GUEST_ID};
use risc0_zkvm::{
    compute_image_id, default_executor,
    sha::{Digest, Digestible},
};
use std::time::Duration;

pub const TX_TIMEOUT: Duration = Duration::from_secs(30);
pub const DCAP_GUEST_ELF: &[u8] = include_bytes!(
    "../../target/riscv-guest/riscv32im-risc0-zkvm-elf/docker/dcap_guest/dcap_guest"
);

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let image_id = compute_image_id(DCAP_GUEST_ELF).unwrap();

    println!("ImageID: {}", &image_id.to_string());

    // TODO: implement methods to properly serialize quotes with its collateral to generate Guest program input...

    // let current_time = std::time::SystemTime::now()
    //     .duration_since(std::time::UNIX_EPOCH)
    //     .unwrap()
    //     .as_secs();

    // let quote = hex::decode(include_str!("../data/quote.hex")).unwrap();

    let input = hex::decode(include_str!("../data/input.hex")).unwrap();

    request_proof_then_verify(&input, &image_id).await;
}

async fn request_proof_then_verify(input: &[u8], image_id: &Digest) {
    let rpc_url = std::env::var("SEPOLIA_URL").unwrap().parse().unwrap();
    let boundless_market_address = address!("69c7943DA0D7e45D44Bd0cE7a2412DCdAe423788");
    let set_verifier_address = address!("Ef0A93B2310d52358F1eCA0C946aD7D25596e7dd");
    let order_stream_url = Some("https://order-stream.beboundless.xyz".parse().unwrap());
    let storage_config = Some(StorageProviderConfig {
        storage_provider: StorageProviderType::Pinata,
        pinata_jwt: Some(std::env::var("PINATA_JWT").unwrap()),
        pinata_api_url: Some(std::env::var("PINATA_API_URL").unwrap().parse().unwrap()),
        ipfs_gateway_url: Some(std::env::var("IPFS_GATEWAY_URL").unwrap().parse().unwrap()),
        s3_access_key: None,
        s3_secret_key: None,
        s3_url: None,
        s3_bucket: None,
        aws_region: None,
        file_path: None,
    });
    let private_key_bytes = hex::decode(std::env::var("PRIVATE_KEY").unwrap()).unwrap();
    let private_key = PrivateKeySigner::from_slice(private_key_bytes.as_slice()).unwrap();

    let boundless_client = ClientBuilder::default()
        .with_rpc_url(rpc_url)
        .with_boundless_market_address(boundless_market_address)
        .with_set_verifier_address(set_verifier_address)
        .with_order_stream_url(order_stream_url)
        .with_storage_provider_config(storage_config.clone())
        .with_private_key(private_key)
        .build()
        .await
        .unwrap();

    let image_url =
        "https://gateway.pinata.cloud/ipfs/QmTEucNxkAmv8UPMGmq34wgwBtLdiYEFPKiwa2AtzwMts1";
    tracing::info!("image url: {}", &image_url);

    let input_builder = InputBuilder::new().write_slice(&input);

    let guest_env = input_builder.clone().build_env().unwrap();
    let guest_env_bytes = guest_env.encode().unwrap();

    let session_info = default_executor()
        .execute(guest_env.try_into().unwrap(), DCAP_GUEST_ELF)
        .unwrap();

    let mcycle_count = session_info
        .segments
        .iter()
        .map(|segment| 1 << segment.po2)
        .sum::<u64>()
        .div_ceil(1_000_000);

    let journal = session_info.journal;

    let request_input = Input::inline(guest_env_bytes.clone());

    let request = ProofRequestBuilder::new()
        .with_image_url(image_url.to_string())
        .with_input(request_input)
        .with_requirements(Requirements::new(
            *image_id,
            Predicate::digest_match(journal.digest()),
        ))
        .with_offer(
            Offer::default()
                .with_min_price_per_mcycle(parse_ether("0.001").unwrap(), mcycle_count)
                .with_max_price_per_mcycle(parse_ether("0.002").unwrap(), mcycle_count)
                .with_timeout(1000),
        )
        .build()
        .unwrap();

    let (request_id, expires_at) = boundless_client.submit_request(&request).await.unwrap();

    println!(
        "Request submitted, id: {}, expires at: {}",
        hex::encode(request_id.to_be_bytes_vec()),
        expires_at
    );

    tracing::info!("Proof Request: {:?}", &request);

    let (journal_bytes, seal) = boundless_client
        .wait_for_request_fulfillment(request_id, Duration::from_secs(5), expires_at)
        .await
        .unwrap();

    let dcap_address = address!("E28ea4E574871CA6A4331d6692bd3DD602Fb4f76");
    let attestation_contract = IAttestation::new(dcap_address, boundless_client.provider().clone());

    let attestation_call = attestation_contract
        .verifyAndAttestWithZKProof(journal_bytes, 1, seal)
        .from(boundless_client.caller());

    let pending_tx = attestation_call.send().await.unwrap();

    let tx_hash = pending_tx
        .with_timeout(Some(TX_TIMEOUT))
        .watch()
        .await
        .expect("failed to confirm tx");

    println!("Tx hash: {}", tx_hash.to_string());
}
