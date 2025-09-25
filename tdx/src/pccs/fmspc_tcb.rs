use super::{remove_prefix_if_found, DEFAULT_RPC_URL, FMSPC_TCB_DAO_ADDRESS};
use anyhow::Result;

use alloy::{
    primitives::{Address, U256},
    providers::ProviderBuilder,
    sol,
};

sol! {
    #[sol(rpc)]
    interface IFmspcTcbDao {
        #[derive(Debug)]
        struct TcbInfoJsonObj {
            string tcbInfoStr;
            bytes signature;
        }

        #[derive(Debug)]
        function getTcbInfo(uint256 tcbType, string calldata fmspc, uint256 version) returns (TcbInfoJsonObj memory tcbObj);
    }
}

pub async fn get_tcb_info(tcb_type: u8, fmspc: &str, version: u32) -> Result<Vec<u8>> {
    let rpc_url = DEFAULT_RPC_URL.parse().expect("Failed to parse RPC URL");
    let provider = ProviderBuilder::new().connect_http(rpc_url);

    let fmspc_tcb_dao_address_slice =
        hex::decode(FMSPC_TCB_DAO_ADDRESS).expect("Invalid address hex");
    let fmspc_tcb_dao_contract =
        IFmspcTcbDao::new(Address::from_slice(&fmspc_tcb_dao_address_slice), &provider);

    let call_builder = fmspc_tcb_dao_contract.getTcbInfo(
        U256::from(tcb_type),
        String::from(fmspc),
        U256::from(version),
    );

    let call_return = call_builder.call().await?;
    let tcb_info_str = call_return.tcbInfoStr;
    let signature_bytes = call_return.signature;

    if tcb_info_str.len() == 0 || signature_bytes.len() == 0 {
        return Err(anyhow::Error::msg(format!(
            "TCBInfo for FMSPC: {}; Version: {} is missing and must be inserted to on-chain pccs",
            fmspc, version
        )));
    }

    let signature = signature_bytes.to_string();

    let ret_str = format!(
        "{{\"tcbInfo\": {}, \"signature\": \"{}\"}}",
        tcb_info_str,
        remove_prefix_if_found(signature.as_str())
    );

    let ret = ret_str.into_bytes();
    Ok(ret)
}
