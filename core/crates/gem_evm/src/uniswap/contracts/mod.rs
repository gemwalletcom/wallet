use alloy_sol_types::sol;

pub mod aero;
pub mod solidly;
pub mod v2;
pub mod v3;
pub mod v4;

sol! {
    #[derive(Debug, PartialEq)]
    interface IUniversalRouter {
        function execute(bytes calldata commands, bytes[] calldata inputs, uint256 deadline) external payable;
    }
}
