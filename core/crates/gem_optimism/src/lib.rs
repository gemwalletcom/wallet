#[cfg(feature = "rpc")]
pub mod gas_oracle;

#[cfg(feature = "rpc")]
pub use gas_oracle::OptimismGasOracle;
