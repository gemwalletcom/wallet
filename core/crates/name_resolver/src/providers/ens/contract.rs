use alloy_sol_types::sol;

sol! {
    interface ENSRegistry {
        function resolver(bytes32 node) external view returns (address);
    }

    interface ENSResolver {
        function addr(bytes32 node) external view returns (address);
    }
}
