use anyhow::{Error, Result};
use risc0_zkvm::{compute_image_id, default_executor, ExecutorEnv};
use std::time::Duration;

use boundless_market::{
    alloy::{
        primitives::{Bytes, U256},
        signers::local::PrivateKeySigner,
        transports::http::reqwest::Url,
    },
    client::Client,
    request_builder::OfferParams,
    storage::storage_provider_from_env,
    Deployment,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ProofType {
    #[value(name = "groth16")]
    Groth16,
    #[value(name = "merkle")]
    MerkleInclusion,
}

impl Default for ProofType {
    fn default() -> Self {
        ProofType::Groth16
    }
}

pub struct ProofRequestConfig {
    pub rpc_url: Url,
    pub private_key: PrivateKeySigner,
    pub deployment: Deployment,
    pub program_url: Option<String>,
    pub proof_type: ProofType,
    pub min_price: Option<U256>,
    pub max_price: Option<U256>,
    pub timeout: Option<u32>,
    pub ramp_up_period: Option<u32>,
}

/// Returns (Journal, Seal) from Boundless
pub async fn request_proof(
    input: &[u8],
    elf: &[u8],
    config: ProofRequestConfig,
) -> Result<(Bytes, Bytes)> {
    let image_id = compute_image_id(elf)?;
    tracing::info!("ImageID: {}", &image_id.to_string());

    let storage_provider = match storage_provider_from_env() {
        Ok(provider) => Some(provider),
        Err(e) => {
            return Err(Error::msg(
                "boundless-error: Storage provider configuration is invalid: ",
            )
            .context(e));
        }
    };

    let client = Client::builder()
        .with_rpc_url(config.rpc_url)
        .with_deployment(config.deployment)
        .with_storage_provider(storage_provider)
        .with_private_key(config.private_key)
        .build()
        .await?;

    // Simulate the execution locally to get the journal
    let env = ExecutorEnv::builder().write_slice(&input).build()?;
    std::env::set_var("RISC0_INFO", "1");
    let session_info = default_executor().execute(env, elf)?;
    tracing::debug!("Session Info: {:?}", &session_info);

    let journal = session_info.journal;

    let mut request_builder = client.new_request().with_stdin(input);

    if let Some(program_url) = config.program_url {
        request_builder = request_builder.with_program_url(program_url.as_str())?;
    } else {
        request_builder = request_builder.with_program(elf.to_vec());
    }

    // Set proof type
    if config.proof_type == ProofType::Groth16 {
        request_builder = request_builder.with_groth16_proof();
    }

    // Only set offer params if user provides them (Boundless handles defaults)
    if config.min_price.is_some()
        || config.max_price.is_some()
        || config.timeout.is_some()
        || config.ramp_up_period.is_some()
    {
        let mut offer_builder = OfferParams::builder();

        if let Some(min_price) = config.min_price {
            offer_builder.min_price(min_price);
        }
        if let Some(max_price) = config.max_price {
            offer_builder.max_price(max_price);
        }
        if let Some(timeout) = config.timeout {
            offer_builder.timeout(timeout);
        }
        if let Some(ramp_up_period) = config.ramp_up_period {
            offer_builder.ramp_up_period(ramp_up_period);
        }

        request_builder = request_builder.with_offer(offer_builder);
    }

    tracing::debug!("Request: {:?}", &request_builder);

    let (request_id, expires_at) = client.submit_onchain(request_builder).await?;

    // Wait for the request to be fulfilled. The market will return the fulfillment.
    tracing::info!("Waiting for request {:x} to be fulfilled", request_id);
    let fulfillment = client
        .wait_for_request_fulfillment(
            request_id,
            Duration::from_secs(5), // check every 5 seconds
            expires_at,
        )
        .await?;
    tracing::info!("Request {:x} fulfilled", request_id);

    Ok((journal.bytes.into(), fulfillment.seal))
}
