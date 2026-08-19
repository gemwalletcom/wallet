#[cfg(feature = "rpc")]
mod contracts;
#[cfg(feature = "rpc")]
mod gas_oracle;

#[cfg(feature = "rpc")]
pub use gas_oracle::OptimismGasOracle;
