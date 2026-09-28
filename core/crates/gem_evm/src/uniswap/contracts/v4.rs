use alloy_sol_types::sol;

sol! {
    interface IPoolManager {
        event Swap(bytes32 indexed id, address indexed sender, int128 amount0, int128 amount1, uint160 sqrtPriceX96, uint128 liquidity, int24 tick, uint24 fee);
    }

    interface IUniswapV4StateView {
        function getSlot0(bytes32 poolId)
            external
            view
            returns (uint160 sqrtPriceX96, int24 tick, uint24 protocolFee, uint24 lpFee);
    }

    type Currency is address;
    #[derive(Debug, PartialEq)]
    struct PoolKey {
        Currency currency0;
        Currency currency1;
        uint24 fee;
        int24 tickSpacing;
        address hooks;
    }

    #[derive(Debug, PartialEq)]
    struct PathKey {
        Currency intermediateCurrency;
        uint24 fee;
        int24 tickSpacing;
        address hooks;
        bytes hookData;
    }

    #[derive(Debug)]
    interface IV4Quoter {
        #[derive(PartialEq)]
        struct QuoteExactSingleParams {
            PoolKey poolKey;
            bool zeroForOne;
            uint128 exactAmount;
            bytes hookData;
        }

        #[derive(PartialEq)]
        struct QuoteExactParams {
            Currency exactCurrency;
            PathKey[] path;
            uint128 exactAmount;
        }

        function quoteExactInputSingle(QuoteExactSingleParams memory params)
            external
            returns (uint256 amountOut, uint256 gasEstimate);

        function quoteExactInput(QuoteExactParams memory params)
            external
            returns (uint256 amountOut, uint256 gasEstimate);

        function quoteExactOutputSingle(QuoteExactSingleParams memory params)
            external
            returns (uint256 amountIn, uint256 gasEstimate);

        function quoteExactOutput(QuoteExactParams memory params)
            external
            returns (uint256 amountIn, uint256 gasEstimate);
    }

    #[derive(Debug)]
    interface IV4Router {
        #[derive(PartialEq)]
        struct ExactInputSingleParams {
            PoolKey poolKey;
            bool zeroForOne;
            uint128 amountIn;
            uint128 amountOutMinimum;
            bytes hookData;
        }

        #[derive(PartialEq)]
        struct ExactInputSingleParamsV2_1 {
            PoolKey poolKey;
            bool zeroForOne;
            uint128 amountIn;
            uint128 amountOutMinimum;
            uint256 minHopPriceX36;
            bytes hookData;
        }

        #[derive(PartialEq)]
        struct ExactInputParams {
            Currency currencyIn;
            PathKey[] path;
            uint128 amountIn;
            uint128 amountOutMinimum;
        }

        #[derive(PartialEq)]
        struct ExactInputParamsV2_1 {
            Currency currencyIn;
            PathKey[] path;
            uint256[] minHopPriceX36;
            uint128 amountIn;
            uint128 amountOutMinimum;
        }

        #[derive(PartialEq)]
        struct ExactOutputSingleParams {
            PoolKey poolKey;
            bool zeroForOne;
            uint128 amountOut;
            uint128 amountInMaximum;
            bytes hookData;
        }

        #[derive(PartialEq)]
        struct ExactOutputSingleParamsV2_1 {
            PoolKey poolKey;
            bool zeroForOne;
            uint128 amountOut;
            uint128 amountInMaximum;
            uint256 minHopPriceX36;
            bytes hookData;
        }

        #[derive(PartialEq)]
        struct ExactOutputParams {
            Currency currencyOut;
            PathKey[] path;
            uint128 amountOut;
            uint128 amountInMaximum;
        }

        #[derive(PartialEq)]
        struct ExactOutputParamsV2_1 {
            Currency currencyOut;
            PathKey[] path;
            uint256[] minHopPriceX36;
            uint128 amountOut;
            uint128 amountInMaximum;
        }
    }
}
