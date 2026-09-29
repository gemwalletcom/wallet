use alloy_sol_types::sol;

sol! {
    interface V3SpokePoolInterface {
        struct V3RelayData {
            address depositor;
            address recipient;
            address exclusiveRelayer;
            address inputToken;
            address outputToken;
            uint256 inputAmount;
            uint256 outputAmount;
            uint256 originChainId;
            uint32 depositId;
            uint32 fillDeadline;
            uint32 exclusivityDeadline;
            bytes message;
        }

        event V3FundsDeposited(
            address inputToken,
            address outputToken,
            uint256 inputAmount,
            uint256 outputAmount,
            uint256 indexed destinationChainId,
            uint32 indexed depositId,
            uint32 quoteTimestamp,
            uint32 fillDeadline,
            uint32 exclusivityDeadline,
            address indexed depositor,
            address recipient,
            address exclusiveRelayer,
            bytes message
        );

        event FundsDeposited(
            bytes32 inputToken,
            bytes32 outputToken,
            uint256 inputAmount,
            uint256 outputAmount,
            uint256 indexed destinationChainId,
            uint256 indexed depositId,
            uint32 quoteTimestamp,
            uint32 fillDeadline,
            uint32 exclusivityDeadline,
            bytes32 indexed depositor,
            bytes32 recipient,
            bytes32 exclusiveRelayer,
            bytes message
        );

        struct RelayExecutionInfo {
            bytes32 updatedRecipient;
            bytes32 updatedMessage;
            uint256 updatedOutputAmount;
            uint8 fillType;
        }

        event FilledRelay(
            bytes32 inputToken,
            bytes32 outputToken,
            uint256 inputAmount,
            uint256 outputAmount,
            uint256 indexed originChainId,
            uint256 indexed depositId,
            uint256 destinationChainId,
            uint32 quoteTimestamp,
            uint32 fillDeadline,
            bytes32 indexed depositor,
            bytes32 recipient,
            bytes32 exclusiveRelayer,
            bytes32 relayer,
            bytes32 settlementContract,
            RelayExecutionInfo relayExecutionInfo
        );

        function getCurrentTime() public view virtual returns (uint256);
        function fillStatuses(bytes32 relayHash) external view returns (uint8);

        function depositV3(
            address depositor,
            address recipient,
            address inputToken,
            address outputToken,
            uint256 inputAmount,
            uint256 outputAmount,
            uint256 destinationChainId,
            address exclusiveRelayer,
            uint32 quoteTimestamp,
            uint32 fillDeadline,
            uint32 exclusivityDeadline,
            bytes calldata message
        ) external payable;

        function fillV3Relay(V3RelayData calldata relayData, uint256 repaymentChainId) external;
    }
}
