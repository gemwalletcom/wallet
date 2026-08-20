use alloy_sol_types::sol;

sol! {
    interface IGasPriceOracle {
        function getL1Fee(bytes memory data) external view returns (uint256);
    }
}
