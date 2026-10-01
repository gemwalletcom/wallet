use alloy_sol_types::sol;

sol! {
    interface HubPoolInterface {
        struct PooledToken {
            address lpToken;
            bool isEnabled;
            uint32 lastLpFeeUpdate;
            int256 utilizedReserves;
            uint256 liquidReserves;
            uint256 undistributedLpFees;
        }

        function paused() external view returns (bool);
        function sync(address l1Token) public override nonReentrant;
        function getCurrentTime() public view returns (uint256);
        function pooledTokens(address l1Token) external view returns (PooledToken memory);
        function liquidityUtilizationCurrent(address l1Token) external returns (uint256);
        function liquidityUtilizationPostRelay(address l1Token, uint256 relayedAmount) external returns (uint256);
    }
}
