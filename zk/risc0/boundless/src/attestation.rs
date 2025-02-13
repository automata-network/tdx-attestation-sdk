use boundless_market::alloy::sol;

sol! {
    #[sol(rpc)]
    interface IAttestation {
        function verifyAndAttestWithZKProof(bytes calldata journal, uint8 zkCoProcessor, bytes calldata seal) returns (bool success, bytes memory output);
    }
}