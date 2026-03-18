use dcap_rs::types::quote::Quote;
use tdx::utils::get_pck_fmspc_and_issuer;
use tdx::Tdx;

fn main() {
    // Initialise a TDX object
    let tdx = Tdx::new();

    // Retrieve an attestation report with default options passed to the hardware device
    let (report_raw, _) = tdx.get_attestation_report_raw().unwrap();
    let report = Quote::read(&mut report_raw.as_slice()).unwrap();

    let (fmspc, _) = get_pck_fmspc_and_issuer(&report).unwrap();
    println!("FMSPC: {:?}", fmspc.to_uppercase());
    if report.header.tee_type == 0 {
        println!("Platform: SGX");
    } else {
        println!("Platform: TDX");
    }
    println!("Version: {}", report.header.version);
}
