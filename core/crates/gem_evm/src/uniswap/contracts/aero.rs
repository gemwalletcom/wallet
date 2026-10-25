use alloy_primitives::U256;
use alloy_sol_types::sol;
use primitives::swap::HUNDRED_PERCENT_IN_BPS;

pub const CL_SWAP_EXACT_IN_COMMAND: u8 = 0x00;
pub const SWEEP_COMMAND: u8 = 0x04;
pub const TRANSFER_COMMAND: u8 = 0x05;
pub const WRAP_ETH_COMMAND: u8 = 0x0b;
pub const UNWRAP_WETH_COMMAND: u8 = 0x0c;
pub const BALANCE_CHECK_COMMAND: u8 = 0x0e;
pub const FUND_ERC20_COMMAND: u8 = 0x0f;
const MAX_PIPS: u32 = 1_000_000;

sol! {
    interface IMetaquoter {
        function quoteExactInput(address[] calldata pools, address tokenIn, uint256 amountIn) external returns (uint256 amountOut, uint160[] memory sqrtPriceX96AfterList, uint32[] memory initializedTicksCrossedList);
    }

    #[derive(Debug, PartialEq)]
    interface IMetarouter {
        function execute(bytes calldata commands, bytes[] calldata inputs, uint256 deadline) external payable;
    }

    #[derive(Debug, PartialEq)]
    enum SpendMode {
        Amount,
        Pips,
    }

    #[derive(Debug, PartialEq)]
    struct BalanceSpend {
        SpendMode mode;
        uint256 value;
    }

    #[derive(Debug, PartialEq)]
    struct SwapExactIn {
        address[] pools;
        address tokenIn;
        BalanceSpend amountIn;
        bool payerIsUser;
        uint256 minAmountOut;
        address recipient;
    }

    #[derive(Debug, PartialEq)]
    struct Transfer {
        address token;
        address recipient;
        BalanceSpend spend;
    }

    #[derive(Debug, PartialEq)]
    struct Sweep {
        address token;
        address recipient;
        uint256 minAmount;
    }

    #[derive(Debug, PartialEq)]
    struct FundErc20 {
        address token;
        BalanceSpend spend;
    }

    #[derive(Debug, PartialEq)]
    struct UnwrapWeth {
        address recipient;
        BalanceSpend spend;
    }

    #[derive(Debug, PartialEq)]
    struct BalanceCheck {
        address asset;
        address owner;
        uint256 minBalance;
    }
}

impl BalanceSpend {
    pub fn amount(value: U256) -> Self {
        Self { mode: SpendMode::Amount, value }
    }

    pub fn share(bps: u32) -> Self {
        Self {
            mode: SpendMode::Pips,
            value: U256::from(bps) * U256::from(MAX_PIPS / HUNDRED_PERCENT_IN_BPS),
        }
    }

    pub fn all() -> Self {
        Self::share(HUNDRED_PERCENT_IN_BPS)
    }
}
