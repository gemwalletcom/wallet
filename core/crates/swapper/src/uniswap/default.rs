use crate::{Swapper, alien::RpcProvider};
use std::sync::Arc;

use super::universal_router::{new_aerodrome, new_oku, new_pancakeswap, new_uniswap_v3, new_uniswap_v4, new_wagmi};

pub fn boxed_uniswap_v3(rpc_provider: Arc<dyn RpcProvider>) -> Box<dyn Swapper> {
    Box::new(new_uniswap_v3(rpc_provider))
}

pub fn boxed_pancakeswap(rpc_provider: Arc<dyn RpcProvider>) -> Box<dyn Swapper> {
    Box::new(new_pancakeswap(rpc_provider))
}

pub fn boxed_aerodrome(rpc_provider: Arc<dyn RpcProvider>) -> Box<dyn Swapper> {
    Box::new(new_aerodrome(rpc_provider))
}

pub fn boxed_oku(rpc_provider: Arc<dyn RpcProvider>) -> Box<dyn Swapper> {
    Box::new(new_oku(rpc_provider))
}

pub fn boxed_wagmi(rpc_provider: Arc<dyn RpcProvider>) -> Box<dyn Swapper> {
    Box::new(new_wagmi(rpc_provider))
}

pub fn boxed_uniswap_v4(rpc_provider: Arc<dyn RpcProvider>) -> Box<dyn Swapper> {
    Box::new(new_uniswap_v4(rpc_provider))
}
