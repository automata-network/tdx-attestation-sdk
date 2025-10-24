use alloy_sol_types::{SolType, sol};
use dcap_rs::types::collateral::Collateral;
use der::Encode;
use std::{fs, path::PathBuf};

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

/// Prints the following collateral
/// X509 encoded as DER bytes
/// JSON encoded as strings
pub fn save_collateral_to_output(collateral: &Collateral, out_dir: &PathBuf) {
    if !out_dir.exists() {
        fs::create_dir_all(out_dir).expect("Failed to create output directory");
    }
    
    // print qe identity JSON
    let qe_identity_path = out_dir.join("identity.json");
    let qe_identity_json = serde_json::to_string(&collateral.qe_identity)
        .expect("Failed to serialize QE Identity to JSON");
    fs::write(qe_identity_path, qe_identity_json)
        .expect("Failed to write QE Identity JSON to file");

    // print TCBInfo JSON
    let tcb_info_path = out_dir.join("tcb_info.json");
    let tcb_info_json =
        serde_json::to_string(&collateral.tcb_info).expect("Failed to serialize TCB Info to JSON");
    fs::write(tcb_info_path, tcb_info_json).expect("Failed to write TCB Info JSON to file");

    // print root crl der
    let root_crl_path = out_dir.join("root_ca_crl.der");
    let root_crl_byte = &collateral
        .root_ca_crl
        .to_der()
        .expect("Failed to encode Root CA CRL as DER");
    fs::write(root_crl_path, root_crl_byte).expect("Failed to write Root CA CRL DER to file");

    // print pck crl der
    let pck_crl_path = out_dir.join("pck_crl.der");
    let pck_crl_byte = &collateral
        .pck_crl
        .to_der()
        .expect("Failed to encode PCK CRL as DER");
    fs::write(pck_crl_path, pck_crl_byte).expect("Failed to write PCK CRL DER to file");
}

/// Loads an ELF file from the specified path.
pub fn load_elf(path: &str) -> Vec<u8> {
    fs::read(path).unwrap_or_else(|err| {
        panic!("Failed to load ELF file from {}: {}", path, err);
    })
}
