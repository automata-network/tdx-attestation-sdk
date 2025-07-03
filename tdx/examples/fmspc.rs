use tdx::Tdx;
use tdx::utils::get_pck_fmspc_and_issuer;

fn main() {
    // Initialise a TDX object
    let tdx = Tdx::new();

    // Retrieve an attestation report with default options passed to the hardware device
    let (report, _) = tdx.get_attestation_report().unwrap();
    // println!("Attestation Report: {:?}", report);

    let (fmspc, _) = get_pck_fmspc_and_issuer(&report);
    println!("FMSPC: {:?}", fmspc.to_uppercase());
}