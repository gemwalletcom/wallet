use alloy_sol_types::sol;

sol! {
    interface IERC677 {
        function transferAndCall(address to, uint256 value, bytes data) external returns (bool);
    }
}
