use alloy_sol_types::sol;

sol! {
    interface ITempoFeeManager {
        function setUserToken(address token) external;
        function userTokens(address user) external view returns (address);
    }

    interface ITIP20 {
        function currency() external view returns (string memory);
    }
}
