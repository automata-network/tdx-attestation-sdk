use core::GuestInput;
use dcap_rs::types::collateral::Collateral;
use pccs_reader_rs::{dotenvy, tcb_pem::generate_tcb_issuer_chain_pem};

use clap::Parser;
use std::fs::{read, read_to_string, write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "DcapInputGenerator")]
#[command(version = "1.1.0")]
#[command(about = "Generates the raw input for DCAP RiscZero Guest Program")]
pub struct Cli {
    /// The path to the raw quote binary file
    #[arg(long = "quote-path", env = "QUOTE_PATH")]
    pub quote_path: PathBuf,

    /// The path to the QEIdentity JSON string file
    #[arg(long = "qe-identity-path", env = "QE_IDENTITY_PATH")]
    pub qe_identity_path: PathBuf,

    /// The path to the FMSPC TCBInfo JSON string file
    #[arg(long = "tcb-info-path", env = "FMSPC_TCB_PATH")]
    pub tcb_info_path: PathBuf,

    /// The path to the TCB Signing CA binary encoded in DER
    #[arg(long = "tcb-signing-path", env = "TCB_SIGNING_CA_DER_PATH")]
    pub tcb_signing_path: PathBuf,

    /// The path to the Root CA binary encoded in DER
    #[arg(long = "root-ca-path", env = "ROOT_CA_DER_PATH")]
    pub root_ca_path: PathBuf,

    /// The path to the Root CA CRL binary encoded in DER
    #[arg(long = "root-crl-path", env = "ROOT_CA_CRL_DER_PATH")]
    pub root_crl_path: PathBuf,

    /// The path to the PCK CRL binary encoded in DER
    #[arg(long = "pck-crl-path", env = "PCK_CRL_DER_PATH")]
    pub pck_crl_path: PathBuf,

    /// Optional: Overrides the timestamp for the input
    /// If not provided, the current system time will be used
    #[arg(short = 't', long = "timestamp")]
    pub timestamp: Option<u64>,
}

fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    // read all the files
    let raw_quote =
        read(get_path_relative_to_manifest(&cli.quote_path)).expect("Failed to read quote file");
    let qe_identity = read_to_string(get_path_relative_to_manifest(&cli.qe_identity_path))
        .expect("Failed to read QE Identity file");
    let tcb_info = read_to_string(get_path_relative_to_manifest(&cli.tcb_info_path))
        .expect("Failed to read TCB Info file");
    let tcb_signing_ca = read(get_path_relative_to_manifest(&cli.tcb_signing_path))
        .expect("Failed to read TCB Signing CA DER file");
    let root_ca = read(get_path_relative_to_manifest(&cli.root_ca_path))
        .expect("Failed to read Root CA DER file");
    let root_ca_crl = read(get_path_relative_to_manifest(&cli.root_crl_path))
        .expect("Failed to read Root CA CRL DER file");
    let pck_crl = read(get_path_relative_to_manifest(&cli.pck_crl_path))
        .expect("Failed to read PCK CRL DER file");

    // generate the TCB issuer chain PEM
    let tcb_issuer_chain_pem =
        generate_tcb_issuer_chain_pem(tcb_signing_ca.as_slice(), root_ca.as_slice()).unwrap();

    let collateral = Collateral::new(
        root_ca_crl.as_slice(),
        pck_crl.as_slice(),
        tcb_issuer_chain_pem.as_bytes(),
        tcb_info.as_str(),
        qe_identity.as_str(),
    )
    .unwrap();

    let current_time = cli.timestamp.unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });

    let guest_input = GuestInput {
        raw_quote: raw_quote.to_vec(),
        collateral: collateral,
        timestamp: current_time,
    };

    log::debug!("Guest Input: {:?}", guest_input);

    let write_path = PathBuf::from("input.bin");
    write(write_path, guest_input.sol_abi_encode()).expect("Failed to write input.bin");
}

const CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");
fn get_path_relative_to_manifest(path: &PathBuf) -> PathBuf {
    let mut full_path = PathBuf::from(CARGO_MANIFEST_DIR);
    full_path.push(path);
    full_path
}
