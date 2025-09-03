use clap::{Args, Parser, Subcommand, ValueEnum};
use std::fs::read_to_string;
use std::path::PathBuf;

const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");
const DEFAULT_QUOTE_PATH: &str = "./data/quote.hex";

#[derive(Parser)]
#[command(name = "DcapSP1App")]
#[command(version = "1.1.0")]
#[command(about = "Gets SP1 Proof for DCAP Quote Verification and submits on-chain")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Fetches proof from SP1 and sends them on-chain to verify DCAP quote
    Prove(DcapArgs),

    /// De-serializes and prints information about the Output
    Deserialize(OutputArgs),
}

/// Enum representing the available proof systems
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum ProofSystem {
    Groth16,
    Plonk,
}

/// Enum representing the network prover modes
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum NetworkProverMode {
    /// Uses prover network on mainnet
    Auction,
    /// Uses Succinct Labs on-demand prover
    Hosted,
    /// Uses an already existing agreement with a fulfiller
    Reserved
}

#[derive(Args)]
pub struct DcapArgs {
    /// The input quote provided as a hex string, this overwrites the --quote-path argument
    #[arg(short = 'q', long = "quote-hex")]
    pub quote_hex: Option<String>,

    /// Optional: The path to a quote.hex file. Default: /data/quote.hex or overwritten by the --quote-hex argument if provided.
    #[arg(short = 'p', long = "quote-path")]
    pub quote_path: Option<PathBuf>,

    #[arg(
        short = 's',
        long = "prove-system",
        value_enum,
        default_value = "groth16"
    )]
    pub proof_system: Option<ProofSystem>,

    #[arg(
        short = 'n',
        long = "network-prover-mode",
        value_enum,
        default_value = "auction"
    )]
    pub network_prover_mode: Option<NetworkProverMode>,

    /// Optional: Locally verify the proof after it is generated
    #[arg(short = 'v', long = "verify")]
    pub verify: bool,
}

#[derive(Args)]
pub struct OutputArgs {
    #[arg(short = 'o', long = "output")]
    pub output: String,
}

pub fn get_quote(path: &Option<PathBuf>, hex: &Option<String>) -> Vec<u8> {
    let error_msg: &str = "Failed to read quote from the provided path";
    match hex {
        Some(h) => {
            let quote_hex = hex::decode(h).expect(error_msg);
            quote_hex
        }
        _ => match path {
            Some(p) => {
                let quote_string = read_to_string(p).expect(error_msg);
                let processed = remove_prefix_if_found(&quote_string);
                let quote_hex = hex::decode(processed).expect(error_msg);
                quote_hex
            }
            _ => {
                let default_path = PathBuf::from(format!(
                    "{}/{}",
                    MANIFEST_DIR,
                    DEFAULT_QUOTE_PATH
                ));
                let quote_string = read_to_string(default_path).expect(error_msg);
                let processed = remove_prefix_if_found(&quote_string);
                let quote_hex = hex::decode(processed).expect(error_msg);
                quote_hex
            }
        },
    }
}

pub fn remove_prefix_if_found(h: &str) -> &str {
    if h.starts_with("0x") {
        &h[2..]
    } else {
        &h
    }
}
