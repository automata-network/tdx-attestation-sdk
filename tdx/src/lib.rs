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

use crate::utils::der_to_pem_bytes;

#[derive(Debug, Clone, Default)]
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
        let device = device::Device::with_default_options()?;
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

    pub async fn verify_attestation_report_raw(&self, raw_quote: &[u8]) -> Result<()> {
        let collaterals = self.get_collaterals(raw_quote).await?;
        let quote = Quote::read(&mut &*raw_quote)?;
        verify_dcap_quote(SystemTime::now(), collaterals, quote)?;
        Ok(())
    }

    /// Retrieve the collaterals required to verify the attestation report.
    pub async fn get_collaterals(&self, raw_quote: &[u8]) -> Result<Collateral> {
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
        let collaterals = find_missing_collaterals_from_quote(
            &provider, None, // deployment_version - uses default
            raw_quote, false, // don't print to disk
            None,  // tcb_eval_num
        )
        .await
        .map_err(|e| match e {
            CollateralError::Missing(report) => {
                TdxError::Http(format!("Missing collaterals: {report:?}"))
            }
            CollateralError::Validation(msg) => {
                TdxError::Http(format!("Validation error: {}", msg))
            }
        })?;

        // Convert library's Collaterals to dcap-rs Collateral
        // The library returns DER-encoded certs, dcap-rs expects PEM for the cert chain
        let mut combined_pem =
            der_to_pem_bytes(&collaterals.tcb_signing_ca);
        combined_pem.extend_from_slice(&der_to_pem_bytes(&collaterals.root_ca));

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
    use std::ptr::copy_nonoverlapping;
    use std::sync::{LazyLock, Mutex};

    use super::device::DeviceOptions;
    use super::Tdx;

    /// Error codes returned by C FFI functions.
    pub const TDX_OK: i32 = 0;
    pub const TDX_ERR_NULL_POINTER: i32 = -1;
    pub const TDX_ERR_BUFFER_TOO_SMALL: i32 = -2;
    pub const TDX_ERR_NO_REPORT: i32 = -3;
    pub const TDX_ERR_ATTESTATION_FAILED: i32 = -4;
    pub const TDX_ERR_LOCK_POISONED: i32 = -5;

    static ATTESTATION_REPORT: LazyLock<Mutex<Vec<u8>>> = LazyLock::new(|| Mutex::new(Vec::new()));
    static VAR_DATA: LazyLock<Mutex<Vec<u8>>> = LazyLock::new(|| Mutex::new(Vec::new()));

    /// Helper to store report and var_data into the global statics.
    /// Returns the report length on success, or a negative error code on failure.
    fn store_report(report_bytes: Vec<u8>, var_data: Option<Vec<u8>>) -> i32 {
        let report_len = match i32::try_from(report_bytes.len()) {
            Ok(len) => len,
            Err(_) => {
                eprintln!("tdx: report length exceeds i32::MAX");
                return TDX_ERR_ATTESTATION_FAILED;
            }
        };
        match ATTESTATION_REPORT.lock() {
            Ok(mut t) => *t = report_bytes,
            Err(e) => {
                eprintln!("tdx: attestation report lock poisoned: {e}");
                return TDX_ERR_LOCK_POISONED;
            }
        }
        // Always update var_data: clear it when None to avoid stale data from previous calls
        match VAR_DATA.lock() {
            Ok(mut t) => *t = var_data.unwrap_or_default(),
            Err(e) => {
                eprintln!("tdx: var data lock poisoned: {e}");
                return TDX_ERR_LOCK_POISONED;
            }
        }
        report_len
    }

    /// Generate the attestation report with default settings.
    ///
    /// Returns the size of the report on success (>= 0), which you can use to malloc
    /// a buffer of suitable size before calling `tdx_get_attestation_report_raw()`.
    /// Returns a negative error code on failure.
    #[unsafe(no_mangle)]
    pub extern "C" fn tdx_generate_attestation_report() -> i32 {
        let tdx = Tdx::new();
        let (report_bytes, var_data) = match tdx.get_attestation_report_raw() {
            Ok(r) => r,
            Err(e) => {
                eprintln!("tdx: failed to get attestation report: {e}");
                return TDX_ERR_ATTESTATION_FAILED;
            }
        };
        store_report(report_bytes, var_data)
    }

    /// Generate the attestation report with custom report_data (64 bytes).
    ///
    /// `report_data` must point to a buffer of at least 64 bytes.
    /// Returns the size of the report on success (>= 0), or a negative error code on failure.
    #[unsafe(no_mangle)]
    pub extern "C" fn tdx_generate_attestation_report_with_options(report_data: *const u8) -> i32 {
        if report_data.is_null() {
            eprintln!("tdx: report_data is null");
            return TDX_ERR_NULL_POINTER;
        }
        let tdx = Tdx::new();
        let mut rust_report_data: [u8; 64] = [0; 64];
        unsafe {
            copy_nonoverlapping(report_data, rust_report_data.as_mut_ptr(), 64);
        }
        let device_options = DeviceOptions {
            report_data: Some(rust_report_data),
        };
        let (report_bytes, var_data) =
            match tdx.get_attestation_report_raw_with_options(device_options) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("tdx: failed to get attestation report with options: {e}");
                    return TDX_ERR_ATTESTATION_FAILED;
                }
            };
        store_report(report_bytes, var_data)
    }

    /// Copy the attestation report into the provided buffer.
    ///
    /// `buf` must point to a buffer of at least `buf_len` bytes.
    /// Call `tdx_generate_attestation_report()` first to obtain the required size.
    ///
    /// Returns the number of bytes written on success (>= 0), or a negative error code:
    /// - `TDX_ERR_NULL_POINTER` if `buf` is null
    /// - `TDX_ERR_BUFFER_TOO_SMALL` if `buf_len` is smaller than the report
    /// - `TDX_ERR_NO_REPORT` if no report has been generated yet
    /// - `TDX_ERR_LOCK_POISONED` if the internal mutex is poisoned
    #[unsafe(no_mangle)]
    pub extern "C" fn tdx_get_attestation_report_raw(buf: *mut u8, buf_len: usize) -> i32 {
        if buf.is_null() {
            eprintln!("tdx: tdx_get_attestation_report_raw: buf is null");
            return TDX_ERR_NULL_POINTER;
        }
        let bytes = match ATTESTATION_REPORT.lock() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("tdx: attestation report lock poisoned: {e}");
                return TDX_ERR_LOCK_POISONED;
            }
        };
        if bytes.is_empty() {
            eprintln!(
                "tdx: no attestation report found, call tdx_generate_attestation_report() first"
            );
            return TDX_ERR_NO_REPORT;
        }
        if buf_len < bytes.len() {
            eprintln!("tdx: buffer too small ({buf_len} < {})", bytes.len());
            return TDX_ERR_BUFFER_TOO_SMALL;
        }
        unsafe {
            copy_nonoverlapping(bytes.as_ptr(), buf, bytes.len());
        }
        bytes.len() as i32 // safe: checked via buf_len which is usize from C caller
    }

    /// Retrieve the length of var_data.
    ///
    /// Call this only after `tdx_generate_attestation_report()`.
    /// Returns 0 if var_data is empty, or a negative error code on failure.
    #[unsafe(no_mangle)]
    pub extern "C" fn tdx_get_var_data_len() -> i32 {
        match VAR_DATA.lock() {
            Ok(t) => t.len() as i32,
            Err(e) => {
                eprintln!("tdx: var data lock poisoned: {e}");
                TDX_ERR_LOCK_POISONED
            }
        }
    }

    /// Copy var_data into the provided buffer.
    ///
    /// `buf` must point to a buffer of at least `buf_len` bytes.
    /// Call `tdx_get_var_data_len()` first to obtain the required size.
    ///
    /// Returns the number of bytes written on success (>= 0), or a negative error code.
    #[unsafe(no_mangle)]
    pub extern "C" fn tdx_get_var_data(buf: *mut u8, buf_len: usize) -> i32 {
        if buf.is_null() {
            eprintln!("tdx: tdx_get_var_data: buf is null");
            return TDX_ERR_NULL_POINTER;
        }
        let bytes = match VAR_DATA.lock() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("tdx: var data lock poisoned: {e}");
                return TDX_ERR_LOCK_POISONED;
            }
        };
        if bytes.is_empty() {
            eprintln!("tdx: no var data found, call tdx_generate_attestation_report() first");
            return TDX_ERR_NO_REPORT;
        }
        if buf_len < bytes.len() {
            eprintln!("tdx: buffer too small ({buf_len} < {})", bytes.len());
            return TDX_ERR_BUFFER_TOO_SMALL;
        }
        unsafe {
            copy_nonoverlapping(bytes.as_ptr(), buf, bytes.len());
        }
        bytes.len() as i32
    }
}
