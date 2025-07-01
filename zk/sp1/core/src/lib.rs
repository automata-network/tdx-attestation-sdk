use serde::{Deserialize, Serialize};
use dcap_rs::types::collateral::Collateral;

#[derive(Serialize, Deserialize)]
pub struct GuestInput {
    pub collateral: Collateral,
    pub raw_quote: Vec<u8>,
    pub timestamp: u64,
}