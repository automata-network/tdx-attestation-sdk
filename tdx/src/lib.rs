pub mod device;
pub mod error;
pub mod utils;

use alloy::providers::ProviderBuilder;
use automata_dcap_network_registry::Network;
use dcap_rs::types::collateral::Collateral;
use dcap_rs::types::quote::Quote;
use dcap_rs::verify_dcap_quote;
use error::{Result, TdxError};
use pccs_reader_rs::{find_missing_collaterals_from_quote, CollateralError};
use std::time::SystemTime;
use tokio::runtime::Runtime;

use crate::utils::der_to_pem_bytes;

pub struct Tdx;

impl Tdx {
    pub fn new() -> Self {
        Tdx
    }

    /// Retrieve an Attestation Report in raw bytes.
    ///
    /// Returns:
    /// - A tuple containing the attestation report and the optional var data.
    /// - The attestation report is raw bytes that can be used with dcap-rs's QuoteV4::from_bytes().
    /// - The var data is an optional `Vec<u8>` containing the var data.
    /// Var data is only available if the device resides on an Azure Confidential VM.
    /// Var data provided by Azure can be used to verify the contents of the attestation report's report_data
    pub fn get_attestation_report_raw(&self) -> Result<(Vec<u8>, Option<Vec<u8>>)> {
        let device = device::Device::default()?;
        device.get_attestation_report_raw()
    }

    /// Retrieve an Attestation Report (as raw bytes) with options.
    /// When available, users can pass in a 64 byte report data when requesting an attestation report.
    /// This cannot be used on Azure Confidential VM.
    pub fn get_attestation_report_raw_with_options(
        &self,
        options: device::DeviceOptions,
    ) -> Result<(Vec<u8>, Option<Vec<u8>>)> {
        let device = device::Device::new(options)?;
        device.get_attestation_report_raw()
    }

    pub fn verify_attestation_report_raw(&self, raw_quote: &[u8]) -> Result<()> {
        let collaterals = self.get_collaterals(raw_quote)?;
        let quote = Quote::read(&mut &*raw_quote)?;
        verify_dcap_quote(SystemTime::now(), collaterals, quote)?;
        Ok(())
    }

    /// This function verifies the chain of trust for the attestation report.
    pub fn verify_attestation_report(&self, raw_quote: &[u8], report: Quote) -> Result<()> {
        let collaterals = self.get_collaterals(raw_quote)?;
        verify_dcap_quote(SystemTime::now(), collaterals, report)?;
        Ok(())
    }

    /// Retrieve the collaterals required to verify the attestation report.
    pub fn get_collaterals(&self, raw_quote: &[u8]) -> Result<Collateral> {
        let rt = Runtime::new().unwrap();

        // Get network configuration (defaults to automata_testnet)
        let network = Network::default_network(None)
            .ok_or_else(|| TdxError::Http("Failed to get network config".to_string()))?;

        // Get RPC endpoint from network registry
        let rpc_url = network
            .rpc_endpoints
            .first()
            .ok_or_else(|| TdxError::Http("No RPC endpoints available".to_string()))?
            .parse()
            .map_err(|e| TdxError::Http(format!("Failed to parse RPC URL: {}", e)))?;

        let provider = ProviderBuilder::new().connect_http(rpc_url);

        // Fetch collaterals from on-chain PCCS using the library
        let collaterals = rt
            .block_on(find_missing_collaterals_from_quote(
                &provider,
                None,  // deployment_version - uses default
                raw_quote,
                false, // don't print to disk
                None,  // tcb_eval_num
            ))
            .map_err(|e| match e {
                CollateralError::Missing(report) => {
                    TdxError::Http(format!("Missing collaterals: {:?}", report))
                }
                CollateralError::Validation(msg) => TdxError::Http(format!("Validation error: {}", msg)),
            })?;

        // Convert library's Collaterals to dcap-rs Collateral
        // The library returns DER-encoded certs, dcap-rs expects PEM for the cert chain
        let signing_ca_pem = der_to_pem_bytes(&collaterals.tcb_signing_ca);
        let root_ca_pem = der_to_pem_bytes(&collaterals.root_ca);
        let combined_pem = [signing_ca_pem, root_ca_pem].concat();

        Ok(Collateral::new(
            &collaterals.root_ca_crl,
            &collaterals.pck_crl,
            &combined_pem,
            &collaterals.tcb_info,
            &collaterals.qe_identity,
        )?)
    }
}

#[cfg(feature = "clib")]
pub mod c {
    use once_cell::sync::Lazy;
    use std::ptr::copy_nonoverlapping;
    use std::sync::Mutex;

    use super::device::DeviceOptions;
    use super::Tdx;

    static ATTESTATION_REPORT: Lazy<Mutex<Vec<u8>>> = Lazy::new(|| Mutex::new(Vec::new()));
    static VAR_DATA: Lazy<Mutex<Vec<u8>>> = Lazy::new(|| Mutex::new(Vec::new()));

    /// Use this function to generate the attestation report with default settings.
    /// Returns the size of the report, which you can use to malloc a buffer of suitable size
    /// before you call get_attestation_report_raw().
    #[no_mangle]
    pub extern "C" fn tdx_generate_attestation_report() -> usize {
        let tdx = Tdx::new();

        let (report_bytes, var_data) = tdx.get_attestation_report_raw().unwrap();
        let report_len = report_bytes.len();
        let var_data_len = var_data.as_ref().map_or(0, |v| v.len());
        match ATTESTATION_REPORT.lock() {
            Ok(mut t) => {
                *t = report_bytes;
            }
            Err(e) => {
                panic!("Error: {:?}", e);
            }
        }

        if var_data_len > 0 {
            match VAR_DATA.lock() {
                Ok(mut t) => {
                    *t = var_data.unwrap();
                }
                Err(e) => {
                    panic!("Error: {:?}", e);
                }
            }
        }
        report_len
    }

    /// Use this function to generate the attestation report with options.
    /// Returns the size of the report, which you can use to malloc a buffer of suitable size
    /// before you call get_attestation_report_raw().
    #[no_mangle]
    pub extern "C" fn tdx_generate_attestation_report_with_options(report_data: *mut u8) -> usize {
        let tdx = Tdx::new();
        let mut rust_report_data: [u8; 64] = [0; 64];
        unsafe {
            copy_nonoverlapping(report_data, rust_report_data.as_mut_ptr(), 64);
        }
        let device_options = DeviceOptions {
            report_data: Some(rust_report_data),
        };
        let (report_bytes, var_data) = tdx
            .get_attestation_report_raw_with_options(device_options)
            .unwrap();
        let report_len = report_bytes.len();
        let var_data_len = var_data.as_ref().map_or(0, |v| v.len());
        match ATTESTATION_REPORT.lock() {
            Ok(mut t) => {
                *t = report_bytes;
            }
            Err(e) => {
                panic!("Error: {:?}", e);
            }
        }

        if var_data_len > 0 {
            match VAR_DATA.lock() {
                Ok(mut t) => {
                    *t = var_data.unwrap();
                }
                Err(e) => {
                    panic!("Error: {:?}", e);
                }
            }
        }
        report_len
    }

    /// Ensure that tdx_generate_attestation_report() is called first to get the size of buf.
    /// Use this size to malloc enough space for the attestation report that will be transferred.
    #[no_mangle]
    pub extern "C" fn tdx_get_attestation_report_raw(buf: *mut u8) {
        let bytes = match ATTESTATION_REPORT.lock() {
            Ok(t) => t.clone(),
            Err(e) => {
                panic!("Error: {:?}", e);
            }
        };
        if bytes.len() == 0 {
            panic!("Error: No attestation report found! Please call tdx_generate_attestation_report() first.");
        }

        unsafe {
            copy_nonoverlapping(bytes.as_ptr(), buf, bytes.len());
        }
    }

    /// Retrieve the length of var_data. Please call this only after you have called
    /// generate_attestation_report(). If var_data is empty, this function will return 0.
    #[no_mangle]
    pub extern "C" fn tdx_get_var_data_len() -> usize {
        let length = match VAR_DATA.lock() {
            Ok(t) => t.len(),
            Err(e) => {
                panic!("Error: {:?}", e);
            }
        };
        length
    }

    /// Retrieve var_data. Please call this only after you have called
    /// get_var_data_len() to malloc a buffer of an appropriate size.
    #[no_mangle]
    pub extern "C" fn tdx_get_var_data(buf: *mut u8) {
        let bytes = match VAR_DATA.lock() {
            Ok(t) => t.clone(),
            Err(e) => {
                panic!("Error: {:?}", e);
            }
        };
        if bytes.len() == 0 {
            panic!("Error: No var data found! Please call generate_attestation_report() first.");
        }

        unsafe {
            copy_nonoverlapping(bytes.as_ptr(), buf, bytes.len());
        }
    }
}
