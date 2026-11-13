use alloy_primitives::{Address, U256};
use alloy_sol_types::sol;

sol! {
    interface IOkxDexRouter {
        event OrderRecord(address fromToken, address toToken, address sender, uint256 fromAmount, uint256 returnAmount);
        event CommissionFromTokenRecord(address fromTokenAddress, uint256 commissionAmount, address referrerAddress, uint256 commissionRate);
        event CommissionToTokenRecord(address toTokenAddress, uint256 commissionAmount, address referrerAddress, uint256 commissionRate);

        struct BaseRequest {
            uint256 fromToken;
            address toToken;
            uint256 fromTokenAmount;
            uint256 minReturnAmount;
            uint256 deadLine;
        }

        struct RouterPath {
            address[] mixAdapters;
            address[] assetTo;
            uint256[] rawData;
            bytes[] extraData;
            uint256 fromToken;
        }

        struct PMMSwapRequest {
            uint256 pathIndex;
            address payer;
            address fromToken;
            address toToken;
            uint256 fromTokenAmountMax;
            uint256 toTokenAmountMax;
            uint256 salt;
            uint256 deadLine;
            bool isPushOrder;
            bytes extension;
        }

        function smartSwapByOrderId(uint256 orderId, BaseRequest baseRequest, uint256[] batchesAmount, RouterPath[][] batches, PMMSwapRequest[] pmmRequests) external payable returns (uint256 returnAmount);
        function smartSwapTo(uint256 orderId, address receiver, BaseRequest baseRequest, uint256[] batchesAmount, RouterPath[][] batches, PMMSwapRequest[] pmmRequests) external payable returns (uint256 returnAmount);
        function dagSwapByOrderId(uint256 orderId, BaseRequest baseRequest, RouterPath[] paths) external payable returns (uint256 returnAmount);
        function dagSwapTo(uint256 orderId, address receiver, BaseRequest baseRequest, RouterPath[] paths) external payable returns (uint256 returnAmount);
        function swapWrapToWithBaseRequest(uint256 orderId, address receiver, BaseRequest baseRequest) external payable returns (uint256 returnAmount);
        function uniswapV3SwapToWithBaseRequest(uint256 orderId, address receiver, BaseRequest baseRequest, uint256[] pools) external payable returns (uint256 returnAmount);
        function unxswapToWithBaseRequest(uint256 orderId, address receiver, BaseRequest baseRequest, bytes32[] pools) external payable returns (uint256 returnAmount);
        function uniswapV3SwapTo(uint256 receiver, uint256 amount, uint256 minReturn, uint256[] pools) external payable returns (uint256 returnAmount);
        function unxswapByOrderId(uint256 srcToken, uint256 amount, uint256 minReturn, bytes32[] pools) external payable returns (uint256 returnAmount);
        function unxswapTo(uint256 srcToken, uint256 amount, uint256 minReturn, address receiver, bytes32[] pools) external payable returns (uint256 returnAmount);
    }
}

pub struct OkxCommission {
    pub token: Address,
    pub amount: U256,
    pub referrer: Address,
}

impl From<IOkxDexRouter::CommissionFromTokenRecord> for OkxCommission {
    fn from(event: IOkxDexRouter::CommissionFromTokenRecord) -> Self {
        Self {
            token: event.fromTokenAddress,
            amount: event.commissionAmount,
            referrer: event.referrerAddress,
        }
    }
}

impl From<IOkxDexRouter::CommissionToTokenRecord> for OkxCommission {
    fn from(event: IOkxDexRouter::CommissionToTokenRecord) -> Self {
        Self {
            token: event.toTokenAddress,
            amount: event.commissionAmount,
            referrer: event.referrerAddress,
        }
    }
}
