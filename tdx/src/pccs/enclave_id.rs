use anyhow::Result;

use super::{remove_prefix_if_found, DEFAULT_RPC_URL, ENCLAVE_ID_DAO_ADDRESS};
use alloy::{
    primitives::{Address, U256},
    providers::ProviderBuilder,
    sol,
};

sol! {
    #[sol(rpc)]
    interface IEnclaveIdentityDao {
        #[derive(Debug)]
        struct EnclaveIdentityJsonObj {
            string identityStr;
            bytes signature;
        }

        #[derive(Debug)]
        function getEnclaveIdentity(uint256 id, uint256 version) returns (EnclaveIdentityJsonObj memory enclaveIdObj);
    }
}

pub async fn get_enclave_identity(version: u32) -> Result<Vec<u8>> {
    let rpc_url = DEFAULT_RPC_URL.parse().expect("Failed to parse RPC URL");
    let provider = ProviderBuilder::new().connect_http(rpc_url);

    let enclave_id_dao_address_slice =
        hex::decode(ENCLAVE_ID_DAO_ADDRESS).expect("Invalid address hex");

    let enclave_id_dao_contract = IEnclaveIdentityDao::new(
        Address::from_slice(&enclave_id_dao_address_slice),
        &provider,
    );

    // EnclaveIdType::TDQE
    let enclave_id_type_uint256 = U256::from(2);

    let call_builder =
        enclave_id_dao_contract.getEnclaveIdentity(enclave_id_type_uint256, U256::from(version));

    let call_return = call_builder.call().await?;

    let identity_str = call_return.identityStr;
    let signature_bytes = call_return.signature;

    if identity_str.len() == 0 || signature_bytes.len() == 0 {
        return Err(anyhow::Error::msg(format!(
            "QEIdentity for TDX; Version: {} is missing and must be inserted to on-chain pccs",
            version
        )));
    }

    let signature = signature_bytes.to_string();

    let ret_str = format!(
        "{{\"enclaveIdentity\": {}, \"signature\": \"{}\"}}",
        identity_str,
        remove_prefix_if_found(signature.as_str())
    );

    let ret = ret_str.into_bytes();
    Ok(ret)
}
