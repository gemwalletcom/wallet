use alloy_sol_types::sol;

sol! {
    interface AcrossConfigStore {
        function l1TokenConfig(address l1Token) returns (string);
    }
}
