use crate::error::{Result, TdxError};
use dcap_rs::{types::quote::Quote, utils::cert_chain_processor};
use rand::RngCore;
use x509_parser::oid_registry::asn1_rs::{oid, FromDer, OctetString, Oid, Sequence};
use x509_parser::prelude::{parse_x509_pem, X509Certificate};

/// PCK Certificate Authority type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PckCA {
    Platform,
    Processor,
}
/// Generates 64 bytes of random data
/// Always guaranteed to return something (ie, unwrap() can be safely called)
pub fn generate_random_data() -> Option<[u8; 64]> {
    let mut data = [0u8; 64];
    rand::thread_rng().fill_bytes(&mut data);
    Some(data)
}

pub fn der_to_pem_bytes(der_bytes: &[u8]) -> Vec<u8> {
    let pem_struct = pem::Pem::new("CERTIFICATE".to_string(), der_bytes.to_vec());
    pem::encode(&pem_struct).into_bytes()
}

pub fn get_pck_fmspc_and_issuer(quote: &Quote) -> Result<(String, PckCA)> {
    let raw_cert_data = &quote.signature.cert_data.cert_data;

    // Cert Chain:
    // [0]: pck ->
    // [1]: pck ca ->
    // [2]: root ca
    let ranges = cert_chain_processor::find_certificate_ranges(raw_cert_data);
    if ranges.is_empty() {
        return Err(TdxError::Dcap("No certificates found".to_string()));
    }
    let (pck_start, pck_end) = ranges[0];
    let (_, pem_struct) = parse_x509_pem(&raw_cert_data[pck_start..pck_end])
        .map_err(|e| TdxError::X509(format!("x509_parser error: {e}")))?;
    let pck = pem_struct
        .parse_x509()
        .map_err(|e| TdxError::X509(format!("x509 error: {e}")))?;

    let pck_issuer = get_x509_issuer_cn(&pck)?;

    let pck_ca = match pck_issuer.as_str() {
        "Intel SGX PCK Platform CA" => PckCA::Platform,
        "Intel SGX PCK Processor CA" => PckCA::Processor,
        _ => return Err(TdxError::Dcap(format!("Unknown PCK Issuer: {pck_issuer}"))),
    };

    let fmspc_slice = extract_fmspc_from_extension(&pck)?;
    let fmspc = hex::encode(fmspc_slice);

    Ok((fmspc, pck_ca))
}

fn get_x509_issuer_cn<'a>(cert: &'a X509Certificate<'a>) -> Result<String> {
    let issuer = cert.issuer();
    let cn = issuer
        .iter_common_name()
        .next()
        .ok_or_else(|| TdxError::X509("Certificate issuer has no Common Name".to_string()))?;
    let cn_str = cn
        .as_str()
        .map_err(|e| TdxError::X509(format!("Issuer CN is not valid UTF-8: {e}")))?;
    Ok(cn_str.to_string())
}

fn extract_fmspc_from_extension<'a>(cert: &'a X509Certificate<'a>) -> Result<[u8; 6]> {
    let sgx_extensions_bytes = cert
        .get_extension_unique(&oid!(1.2.840 .113741 .1 .13 .1))
        .map_err(|e| TdxError::X509(format!("Duplicate SGX extensions in certificate: {e}")))?
        .ok_or_else(|| TdxError::X509("Certificate missing SGX extensions (OID 1.2.840.113741.1.13.1)".to_string()))?
        .value;

    let (_, sgx_extensions) = Sequence::from_der(sgx_extensions_bytes)
        .map_err(|e| TdxError::X509(format!("Failed to parse SGX extensions sequence: {e}")))?;

    let mut i = sgx_extensions.content.as_ref();

    while !i.is_empty() {
        let (j, current_sequence) = Sequence::from_der(i)
            .map_err(|e| TdxError::X509(format!("Failed to parse SGX extension entry: {e}")))?;
        i = j;
        let (j, current_oid) = Oid::from_der(current_sequence.content.as_ref())
            .map_err(|e| TdxError::X509(format!("Failed to parse SGX extension OID: {e}")))?;
        if current_oid.to_id_string() == "1.2.840.113741.1.13.1.4" {
            let (_, fmspc_bytes) = OctetString::from_der(j)
                .map_err(|e| TdxError::X509(format!("Failed to parse FMSPC octet string: {e}")))?;
            let mut fmspc = [0u8; 6];
            let fmspc_ref = fmspc_bytes.as_ref();
            if fmspc_ref.len() != 6 {
                return Err(TdxError::X509(format!(
                    "FMSPC has unexpected length: {} (expected 6)",
                    fmspc_ref.len()
                )));
            }
            fmspc.copy_from_slice(fmspc_ref);
            return Ok(fmspc);
        }
    }

    Err(TdxError::X509("FMSPC extension (OID 1.2.840.113741.1.13.1.4) not found in certificate".to_string()))
}
