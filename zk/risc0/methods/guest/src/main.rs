#![no_main]
use std::io::Read;

use risc0_zkvm::guest::env::{self};

use dcap_rs::types::{
    collaterals::IntelCollateral, quotes::version_3::QuoteV3, quotes::version_4::QuoteV4,
    VerifiedOutput,
};
use dcap_rs::utils::cert::{hash_crl_keccak256, hash_x509_keccak256};
use dcap_rs::utils::quotes::version_3::verify_quote_dcapv3;
use dcap_rs::utils::quotes::version_4::verify_quote_dcapv4;
use dcap_rs::utils::tcbinfo::{get_tcbinfov2_content_hash, get_tcbinfov3_content_hash};
use dcap_rs::utils::enclave_identity::get_enclave_identityv2_content_hash;

risc0_zkvm::guest::entry!(main);

fn main() {
    // read the values passed from host
    let mut input = Vec::new();
    env::stdin().read_to_end(&mut input).unwrap();

    // TODO: currently current_time does nothing since it can be spoofed by the host
    // we can obtain an attested time from a trusted source that is bound to the input values and verify it

    // deserialize the input
    // read the fixed portion first
    let current_time: u64 = u64::from_le_bytes(input[..8].try_into().unwrap());
    let quote_len = u32::from_le_bytes(input[8..12].try_into().unwrap()) as usize;
    let intel_collaterals_bytes_len =
        u32::from_le_bytes(input[12..16].try_into().unwrap()) as usize;

    // read the variable length fields
    let mut offset = 16 as usize; // timestamp + quote_len + collateral_len
    let quote_slice = &input[offset..offset + quote_len];
    offset += quote_len;
    let intel_collaterals_slice = &input[offset..offset + intel_collaterals_bytes_len];
    offset += intel_collaterals_bytes_len;
    assert!(offset == input.len());

    let intel_collaterals = IntelCollateral::from_bytes(&intel_collaterals_slice);

    // check either only platform or processor crls is provided. not both
    let sgx_platform_crl_is_found = (&intel_collaterals.get_sgx_pck_platform_crl()).is_some();
    let sgx_processor_crl_is_found = (&intel_collaterals.get_sgx_pck_processor_crl()).is_some();
    assert!(
        sgx_platform_crl_is_found != sgx_processor_crl_is_found,
        "platform or processor crls only"
    );

    let verified_output: VerifiedOutput;

    let quote_version = u16::from_le_bytes(input[16..18].try_into().unwrap());
    match quote_version {
        3 => {
            let quote = QuoteV3::from_bytes(&quote_slice);
            verified_output = verify_quote_dcapv3(&quote, &intel_collaterals, current_time);
        }
        4 => {
            let quote = QuoteV4::from_bytes(&quote_slice);
            verified_output = verify_quote_dcapv4(&quote, &intel_collaterals, current_time);
        }
        _ => {
            panic!("Unsupported quote version");
        }
    }

    // write public output to the journal
    let serial_output = verified_output.to_bytes();
    
    let tcb_content_hash = match quote_version {
        3 => {
            let tcb_info_v2 = intel_collaterals.get_tcbinfov2();
            get_tcbinfov2_content_hash(&tcb_info_v2)
        },
        4 => {
            let tcb_info_v3 = intel_collaterals.get_tcbinfov3();
            get_tcbinfov3_content_hash(&tcb_info_v3)
        },
        _ => panic!("Unsupported Quote Version")
    };
    
    let qeidentity = intel_collaterals.get_qeidentityv2();
    let qeidentity_content_hash = get_enclave_identityv2_content_hash(&qeidentity);
    
    let sgx_intel_root_ca_cert_hash =
        hash_x509_keccak256(&intel_collaterals.get_sgx_intel_root_ca());
    
    let sgx_tcb_signing_cert_hash = hash_x509_keccak256(&intel_collaterals.get_sgx_tcb_signing());
    
    let sgx_intel_root_ca_crl_hash =
        hash_crl_keccak256(&intel_collaterals.get_sgx_intel_root_ca_crl().unwrap());

    let sgx_pck_crl;
    if sgx_platform_crl_is_found {
        sgx_pck_crl = intel_collaterals.get_sgx_pck_platform_crl().unwrap();
    } else {
        sgx_pck_crl = intel_collaterals.get_sgx_pck_processor_crl().unwrap();
    }

    let sgx_pck_crl_hash = hash_crl_keccak256(&sgx_pck_crl);

    // the journal output has the following format:
    // serial_output_len (2 bytes)
    // serial_output (VerifiedOutput)
    // current_time (8 bytes)
    // tcbinfo_content_hash
    // qeidentity_content_hash
    // sgx_intel_root_ca_cert_hash
    // sgx_tcb_signing_cert_hash
    // sgx_tcb_intel_root_ca_crl_hash
    // sgx_pck_platform_crl_hash or sgx_pck_processor_crl_hash
    let journal_len = serial_output.len() + 226;
    let mut journal_output: Vec<u8> = Vec::with_capacity(journal_len);
    let output_len: u16 = serial_output.len() as u16;

    journal_output.extend_from_slice(&output_len.to_be_bytes());
    journal_output.extend_from_slice(&serial_output);
    journal_output.extend_from_slice(&current_time.to_be_bytes());
    journal_output.extend_from_slice(&tcb_content_hash);
    journal_output.extend_from_slice(&qeidentity_content_hash);
    journal_output.extend_from_slice(&sgx_intel_root_ca_cert_hash);
    journal_output.extend_from_slice(&sgx_tcb_signing_cert_hash);
    journal_output.extend_from_slice(&sgx_intel_root_ca_crl_hash);
    journal_output.extend_from_slice(&sgx_pck_crl_hash);

    env::commit_slice(&journal_output);
}
