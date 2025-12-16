pub mod enclave_id;
pub mod fmspc_tcb;
pub mod pcs;

// Chain Defaults
pub const DEFAULT_RPC_URL: &str = "https://1rpc.io/ata/testnet";

// PCCS addresses
pub const ENCLAVE_ID_DAO_ADDRESS: &str = "6eE9602b90E8C451FfBCc8d5Dc9C8A3BF0A4fA56";
pub const FMSPC_TCB_DAO_ADDRESS: &str = "62E8Cd513B12F248804123f7ed12A0601B79FBAc";
pub const PCS_DAO_ADDRESS: &str = "B270cD8550DA117E3accec36A90c4b0b48daD342";

pub fn remove_prefix_if_found(h: &str) -> &str {
    if h.starts_with("0x") {
        &h[2..]
    } else {
        &h
    }
}
