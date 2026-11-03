use alloy_sol_types::sol;

sol! {
    interface UniversalResolver {
        error ResolverNotFound(bytes name);
        error UnsupportedResolverProfile(bytes4 selector);
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
