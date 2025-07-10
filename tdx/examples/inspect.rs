use std::path::PathBuf;

use clap::Parser;
use dcap_rs::types::quotes::{QuoteHeader, version_3::QuoteV3, version_4::QuoteV4};
use dcap_rs::utils::cert::{parse_certchain, parse_pem};
use tdx::utils::{extract_fmspc_from_extension, get_pck_fmspc_and_issuer};

#[derive(Parser)]
struct Opt {
    #[clap(long)]
    report: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();
    let report_path = opt.report;
    let report = std::fs::read(&report_path)?;
    let header = QuoteHeader::from_bytes(&report[0..48]);
    if header.version == 3 {
        let quote_v3 = QuoteV3::from_bytes(&report);
        let fmspc = get_pck_fmspc_from_v3_quote(&quote_v3);
        println!("FMSPC: {:?}", fmspc.to_uppercase());
        println!("Platform: SGX");
        println!("Version: V3");
    } else if header.version == 4 {
        let quote_v4 = QuoteV4::from_bytes(&report);
        let (fmspc, _) = get_pck_fmspc_and_issuer(&quote_v4);
        println!("FMSPC: {:?}", fmspc.to_uppercase());
        if quote_v4.header.tee_type == 0 {
            println!("Platform: SGX");
        } else {
            println!("Platform: TDX");
        }
        println!("Version: V4");
    } else {
        eprintln!("Unsupported quote version: {}", header.version);
    }
    Ok(())
}

fn get_pck_fmspc_from_v3_quote(quote: &QuoteV3) -> String {
    let pem = parse_pem(&quote.signature.qe_cert_data.cert_data).expect("Failed to parse cert data");
    let cert_chain = parse_certchain(&pem);
    let pck = &cert_chain[0];
    let fmspc_slice = extract_fmspc_from_extension(pck);
    let fmspc = hex::encode(fmspc_slice);
    fmspc
}
