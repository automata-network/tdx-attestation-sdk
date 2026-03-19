use dcap_rs::types::quote::Quote;
use tdx::Tdx;

#[tokio::main]
async fn main() {
    // Initialise a TDX object
    let tdx = Tdx::new();

    // Retrieve an attestation report with default options passed to the hardware device
    // ================================================================================
    let (report_raw, _) = tdx.get_attestation_report_raw().unwrap();
    let report = Quote::read(&mut report_raw.as_slice()).unwrap();

    println!("Attestation Report: {:?}", report);

    // Verify the attestation report
    // ================================================================================
    tdx.verify_attestation_report_raw(&report_raw)
        .await
        .unwrap();

    println!("Verification successful!");
}
