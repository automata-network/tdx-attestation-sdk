use dcap_rs::types::quote::Quote;
use tdx::Tdx;

fn main() {
    // Initialise a TDX object
    let tdx = Tdx::new();

    // Retrieve an attestation report with default options passed to the hardware device
    // ================================================================================
    let (report_raw, _) = tdx.get_attestation_report_raw().unwrap();
    let report = Quote::read(&mut report_raw.as_slice()).unwrap();

    println!("Attestation Report: {:?}", report);

    // Verify the attestation report
    // Use either function, they both do the same thing, except one takes in bytes
    // while the other takes in a Quote struct.
    // ================================================================================
    // tdx.verify_attestation_report(report).unwrap();
    tdx.verify_attestation_report_raw(&mut report_raw.as_slice())
        .unwrap();

    println!("Verification successful!");
}
