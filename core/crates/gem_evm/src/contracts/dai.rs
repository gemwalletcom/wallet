use alloy_sol_types::sol;

sol! {
    interface IDai {
        function push(address usr, uint256 wad) external;
        function pull(address usr, uint256 wad) external;
        function move(address src, address dst, uint256 wad) external;
    }
}
