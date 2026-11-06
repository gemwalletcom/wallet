use alloy_primitives::Bytes;
use alloy_sol_types::{SolValue, sol};

sol! {
    error HttpError(uint16 status, string message);

    interface UniversalResolver {
        error ResolverNotFound(bytes name);
        error UnsupportedResolverProfile(bytes4 selector);
        error ResolverError(bytes errorData);
        error OffchainLookup(address sender, string[] urls, bytes callData, bytes4 callbackFunction, bytes extraData);

        function resolve(bytes calldata name, bytes calldata data) external view returns (bytes memory, address);
    }

    interface ENSResolver {
        function addr(bytes32 node) external view returns (address);
    }

    interface ENSMulticoinResolver {
        function addr(bytes32 node, uint256 coinType) external view returns (bytes memory);
    }
}

impl UniversalResolver::OffchainLookup {
    pub fn callback(&self, response: Bytes) -> Vec<u8> {
        [self.callbackFunction.as_slice(), &(response, self.extraData.clone()).abi_encode_params()].concat()
    }
}
