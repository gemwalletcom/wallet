use alloy_sol_types::sol;

sol! {
    interface L2Resolver {
        function addr(bytes32 node) external view returns (address);
    }
}
