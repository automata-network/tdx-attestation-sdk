use dcap_rs::types::quote::Quote;
use std::path::PathBuf;

use clap::Parser;
use tdx::utils::get_pck_fmspc_and_issuer;

#[derive(Parser)]
struct Opt {
    #[clap(long)]
    report: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();
    let report_path = opt.report;
    let report = std::fs::read(&report_path)?;
    let report = Quote::read(&mut report.as_slice())?;
    let report_version = u32::from(report.header.version);

    let (fmspc, _) = get_pck_fmspc_and_issuer(&report).unwrap();
    println!("FMSPC: {:?}", fmspc.to_uppercase());
    if report.header.tee_type == 0 {
        println!("Platform: SGX");
    } else {
        println!("Platform: TDX");
    }
    println!("Report Version: V{}", report_version);

    Ok(())
}
