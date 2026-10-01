use alloy_sol_types::sol;

sol! {
    interface IUniswapV3Factory {
        function getPool(address tokenA, address tokenB, uint24 fee) external view returns (address pool);
    }

    #[derive(Debug, PartialEq)]
    interface IQuoterV2 {
        struct QuoteExactInputSingleParams {
            address tokenIn;
            address tokenOut;
            uint256 amountIn;
            uint24 fee;
            uint160 sqrtPriceLimitX96;
        }

        struct QuoteExactOutputSingleParams {
            address tokenIn;
            address tokenOut;
            uint256 amount;
            uint24 fee;
            uint160 sqrtPriceLimitX96;
        }

        function quoteExactInput(bytes memory path, uint256 amountIn)
            external
            returns (
                uint256 amountOut,
                uint160[] memory sqrtPriceX96AfterList,
                uint32[] memory initializedTicksCrossedList,
                uint256 gasEstimate
            );

        function quoteExactInputSingle(QuoteExactInputSingleParams memory params)
            external
            returns (
                uint256 amountOut,
                uint160 sqrtPriceX96After,
                uint32 initializedTicksCrossed,
                uint256 gasEstimate
            );

        function quoteExactOutput(bytes memory path, uint256 amountOut)
            external
            returns (
                uint256 amountIn,
                uint160[] memory sqrtPriceX96AfterList,
                uint32[] memory initializedTicksCrossedList,
                uint256 gasEstimate
            );

        function quoteExactOutputSingle(QuoteExactOutputSingleParams memory params)
            external
            returns (
                uint256 amountIn,
                uint160 sqrtPriceX96After,
                uint32 initializedTicksCrossed,
                uint256 gasEstimate
            );
    }
}
