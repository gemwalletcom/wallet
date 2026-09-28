use alloy_sol_types::sol;

sol! {
    struct Call {
        address target;
        bytes callData;
        uint256 value;
    }

    struct Instructions {
        Call[] calls;
        address fallbackRecipient;
    }
}
