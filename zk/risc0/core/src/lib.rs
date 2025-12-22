use alloy_sol_types::{SolType, sol};
use dcap_rs::types::collateral::Collateral;

#[derive(Debug)]
pub struct GuestInput {
    pub collateral: Collateral,
    pub raw_quote: Vec<u8>,
    pub timestamp: u64,
}

pub type GuestInputSolType = sol!((bytes, bytes, uint64));

impl GuestInput {
    pub fn sol_abi_encode(&self) -> Vec<u8> {
        let collateral_encoded = self.collateral.sol_abi_encode().unwrap();
        GuestInputSolType::abi_encode_params(&(
            collateral_encoded.as_slice(),
            self.raw_quote.as_slice(),
            self.timestamp,
        ))
    }

    pub fn sol_abi_decode(encode: &[u8]) -> Self {
        let (collateral_encoded, raw_quote, timestamp) =
            GuestInputSolType::abi_decode_params(encode).unwrap();
        let collateral = Collateral::sol_abi_decode(&collateral_encoded).unwrap();
        Self {
            collateral,
            raw_quote: raw_quote.to_vec(),
            timestamp,
        }
    }
}
