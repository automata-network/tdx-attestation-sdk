pub mod device;
pub mod error;
pub mod pccs;
pub mod utils;

use dcap_rs::types::collaterals::IntelCollateral;
use dcap_rs::types::quotes::version_4::QuoteV4;
use dcap_rs::utils::quotes::version_4::verify_quote_dcapv4;
use error::{Result, TdxError};
use pccs::enclave_id::get_enclave_identity;
use pccs::fmspc_tcb::get_tcb_info;
use pccs::pcs::{get_certificate_by_id, IPCSDao::CA};
use std::panic;
use tokio::runtime::Runtime;
use utils::get_pck_fmspc_and_issuer;

pub struct Tdx;

impl Tdx {
    pub fn new() -> Self {
        Tdx
    }

    /// Retrieve an Attestation Report.
    ///
    /// Returns:
    /// - A tuple containing the attestation report and the optional var data.
    /// - The attestation report is a `QuoteV4` struct.
    /// - The var data is an optional `Vec<u8>` containing the var data.
    /// Var data is only available if the device resides on an Azure Confidential VM.
    /// Var data provided by Azure can be used to verify the contents of the attestation report's report_data
    pub fn get_attestation_report(&self) -> Result<(QuoteV4, Option<Vec<u8>>)> {
        let device = device::Device::default()?;
        device.get_attestation_report()
    }

    /// Retrieve an Attestation Report with options.
    /// When available, users can pass in a 64 byte report data when requesting an attestation report.
    /// This cannot be used on Azure Confidential VM.
    pub fn get_attestation_report_with_options(
        &self,
        options: device::DeviceOptions,
    ) -> Result<(QuoteV4, Option<Vec<u8>>)> {
        let device = device::Device::new(options)?;
        device.get_attestation_report()
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

    /// This function verifies the chain of trust for the attestation report.
    pub fn verify_attestation_report(&self, report: &QuoteV4) -> Result<()> {
        // First retrieve all the required collaterals.
        let rt = Runtime::new().unwrap();
        let (root_ca, root_ca_crl) = rt.block_on(get_certificate_by_id(CA::ROOT))?;
        if root_ca.is_empty() || root_ca_crl.is_empty() {
            return Err(TdxError::Http("Root CA or CRL is empty".to_string()));
        }

        let (fmspc, pck_type) = get_pck_fmspc_and_issuer(report);
        // tcb_type: 0: SGX, 1: TDX
        // version: TDX uses TcbInfoV3
        let tcb_info = rt.block_on(get_tcb_info(1, &fmspc, 3))?;

        let quote_version = report.header.version;
        let qe_identity = rt.block_on(get_enclave_identity(quote_version as u32))?;

        let (signing_ca, _) = rt.block_on(get_certificate_by_id(CA::SIGNING))?;
        if signing_ca.is_empty() {
            return Err(TdxError::Http("Signing CA is empty".to_string()));
        }

        let (_, pck_crl) = rt.block_on(get_certificate_by_id(pck_type))?;
        if pck_crl.is_empty() {
            return Err(TdxError::Http("PCK CRL is empty".to_string()));
        }

        // Pass all the collaterals into a struct for verifying the quote.
        let current_time = chrono::Utc::now().timestamp() as u64;
        let mut collaterals = IntelCollateral::new();

        collaterals.set_tcbinfo_bytes(&tcb_info);
        collaterals.set_qeidentity_bytes(&qe_identity);
        collaterals.set_intel_root_ca_der(&root_ca);
        collaterals.set_sgx_tcb_signing_der(&signing_ca);
        collaterals.set_sgx_intel_root_ca_crl_der(&root_ca_crl);
        match pck_type {
            CA::PLATFORM => {
                collaterals.set_sgx_platform_crl_der(&pck_crl);
            }
            CA::PROCESSOR => {
                collaterals.set_sgx_processor_crl_der(&pck_crl);
            }
            _ => {
                return Err(TdxError::Http("Unknown PCK Type".to_string()));
            }
        }

        match panic::catch_unwind(|| verify_quote_dcapv4(report, &collaterals, current_time)) {
            Ok(_) => Ok(()),
            Err(e) => Err(TdxError::Dcap(format!("DCAP Error: {:?}", e))),
        }
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

    /// Ensure that generate_attestation_report() is called first to get the size of buf.
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
            panic!("Error: No attestation report found! Please call generate_attestation_report() first.");
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
