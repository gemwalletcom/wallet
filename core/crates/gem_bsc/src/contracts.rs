use alloy_sol_types::sol;

sol! {
    #[derive(Debug, PartialEq)]
    interface IHubReader {
        struct Validator {
            address operatorAddress;
            bool jailed;
            string moniker;
            uint64 commission;
            uint64 apy;
        }

        struct Delegation {
            address delegatorAddress;
            address validatorAddress;
            uint256 amount;
            uint256 shares;
        }

        struct Undelegation {
            address delegatorAddress;
            address validatorAddress;
            uint256 amount;
            uint256 shares;
            uint256 unlockTime;
        }

        function getValidators(uint16 offset, uint16 limit) external view returns (Validator[] memory);
        function getDelegations(address delegator, uint16 offset, uint16 limit) external view returns (Delegation[] memory);
        function getUndelegations(address delegator, uint16 offset, uint16 limit) external view returns (Undelegation[] memory);
    }
}

sol! {
    #[derive(Debug, PartialEq)]
    interface IStakeHub {
        function delegate(address operatorAddress, bool delegateVotePower) external payable;
        function undelegate(address operatorAddress, uint256 shares) external;
        function redelegate(address srcValidator, address dstValidator, uint256 shares, bool delegateVotePower) external;
        function claim(address operatorAddress, uint256 requestNumber) external;
        function claimBatch(address[] calldata operatorAddresses, uint256[] calldata requestNumbers) external;
    }
}
