// Attestation module commented out for now due to name conflict with core crate
// pub mod attestation;
pub mod proving;

pub use proving::{request_proof, ProofRequestConfig, ProofType};
