# Core Features and Providers

These tables describe the current implementation, not a product promise. `Address history` means that `get_transactions_by_address` has a chain-specific implementation; it does not imply that every client constructor wires that implementation. `Simulation` means that the chain overrides the default unsupported simulation behavior. `WalletConnect` means that the chain is advertised for sessions and has request handling.

Availability legend used across comparison tables: ✅ = supported; ❌ = not supported; ➖ = not applicable; 🏗️ = TODO.

Use `➖` only when the capability does not apply to the chain or provider. Use `❌` when it applies but is unavailable.

Review cadence: weekly, and immediately when a referenced Core mapping changes.

| Chain | Type | Address history | Simulation | WalletConnect | Swap | Stake | NFT | DeFi |
| --- | --- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| Bitcoin | Bitcoin | ✅ | ➖ | 🏗️ | ✅ | ➖ | ➖ | ➖ |
| Bitcoin Cash | Bitcoin | ✅ | ➖ | ➖ | ✅ | ➖ | ➖ | ➖ |
| Litecoin | Bitcoin | ✅ | ➖ | ➖ | ✅ | ➖ | ➖ | ➖ |
| Ethereum | EVM | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| BNB Smart Chain | EVM | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Solana | Solana | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Polygon | EVM | ✅ | ✅ | ✅ | ✅ | ➖ | ✅ | ✅ |
| THORChain | Cosmos | ✅ | ❌ | ➖ | ✅ | ➖ | ➖ | ❌ |
| MayaChain | Cosmos | ✅ | ❌ | ➖ | ❌ | ➖ | ➖ | ❌ |
| Cosmos | Cosmos | ✅ | ❌ | ➖ | ✅ | ✅ | ❌ | ❌ |
| Osmosis | Cosmos | ✅ | ❌ | ➖ | ❌ | ✅ | ❌ | ❌ |
| Arbitrum | EVM | ✅ | ✅ | ✅ | ✅ | ➖ | ❌ | ✅ |
| TON | TON | ✅ | ❌ | ✅ | ✅ | ❌ | ✅ | ❌ |
| Tron | Tron | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ |
| Dogecoin | Bitcoin | ✅ | ➖ | ➖ | ✅ | ➖ | ➖ | ➖ |
| Zcash | Bitcoin | ✅ | ➖ | ➖ | ✅ | ➖ | ➖ | ➖ |
| Optimism | EVM | ✅ | ✅ | ✅ | ✅ | ➖ | ❌ | ✅ |
| Aptos | Aptos | ✅ | ❌ | ➖ | ✅ | ✅ | ❌ | ❌ |
| Base | EVM | ✅ | ✅ | ✅ | ✅ | ➖ | ❌ | ✅ |
| Avalanche C-Chain | EVM | ✅ | ✅ | ✅ | ✅ | ➖ | ❌ | ✅ |
| Sui | Sui | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ |
| XRP Ledger | XRP | ✅ | ❌ | ➖ | ✅ | ➖ | ❌ | ❌ |
| opBNB | EVM | ✅ | ✅ | ✅ | ✅ | ➖ | ❌ | ❌ |
| Fantom | EVM | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ |
| Gnosis | EVM | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ |
| Celestia | Cosmos | ✅ | ❌ | ➖ | ❌ | ✅ | ➖ | ❌ |
| Injective | Cosmos | ✅ | ❌ | ➖ | ❌ | ✅ | ❌ | ❌ |
| Sei | Cosmos | ✅ | ❌ | ➖ | ❌ | ✅ | ❌ | ❌ |
| Sei EVM | EVM | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| Manta | EVM | 🏗️ | ✅ | ✅ | ✅ | ➖ | ❌ | ❌ |
| Blast | EVM | ✅ | ✅ | ✅ | ✅ | ➖ | ❌ | ❌ |
| Noble | Cosmos | ✅ | ❌ | ➖ | ❌ | ➖ | ➖ | ❌ |
| ZKsync | EVM | ✅ | ✅ | ✅ | ✅ | ➖ | ❌ | ✅ |
| Linea | EVM | ✅ | ✅ | ✅ | ✅ | ➖ | ❌ | ✅ |
| Mantle | EVM | ✅ | ✅ | ✅ | ✅ | ➖ | ❌ | ❌ |
| Celo | EVM | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ |
| NEAR | NEAR | ✅ | ❌ | ➖ | ✅ | ❌ | ❌ | ❌ |
| World Chain | EVM | ✅ | ✅ | ✅ | ✅ | ➖ | ❌ | ❌ |
| Stellar | Stellar | ✅ | ❌ | ➖ | ✅ | ➖ | ❌ | ❌ |
| Sonic | EVM | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| Algorand | Algorand | ✅ | ❌ | ➖ | ❌ | ❌ | ❌ | ❌ |
| Polkadot | Polkadot | ✅ | ❌ | ➖ | ❌ | ❌ | ❌ | ❌ |
| Plasma | EVM | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| Cardano | Cardano | ✅ | ❌ | ➖ | ✅ | ❌ | ❌ | ❌ |
| Abstract | EVM | ✅ | ✅ | ✅ | ✅ | ➖ | ❌ | ❌ |
| Berachain | EVM | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| Ink | EVM | ✅ | ✅ | ✅ | ✅ | ➖ | ❌ | ❌ |
| Unichain | EVM | ✅ | ✅ | ✅ | ✅ | ➖ | ❌ | ❌ |
| Hyperliquid | EVM | ✅ | ✅ | ✅ | ✅ | ➖ | ❌ | ❌ |
| HyperCore | HyperCore | ✅ | ❌ | ➖ | ✅ | ✅ | ➖ | ❌ |
| Monad | EVM | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ |
| X Layer | EVM | ✅ | ✅ | ✅ | ✅ | ➖ | ❌ | ❌ |
| Robinhood Chain | EVM | ✅ | ✅ | ✅ | ✅ | ➖ | ❌ | ❌ |
| Stable | EVM | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ |

<sub>Reviewed 2026-07-21. Sources: [chain list](../crates/primitives/src/chain.rs), [feature configuration](../crates/primitives/src/chain_config.rs), [address-history contract](../crates/chain_traits/src/lib.rs), [simulation implementations](../crates/gem_evm/src/provider/simulation.rs), [Solana](../crates/gem_solana/src/provider/simulation.rs), [Sui](../crates/gem_sui/src/provider/simulation.rs), [Tron](../crates/gem_tron/src/provider/simulation.rs), [WalletConnect chain configuration](../gemstone/src/config/wallet_connect.rs), and [WalletConnect request handlers](../crates/gem_wallet_connect/src/request_handler/mod.rs).</sub>

## WalletConnect

The status is based on both the chains returned by Core configuration and the methods accepted by the Core request dispatcher. Method-set links open the chain-specific handler.

In this section, `➖` means WalletConnect does not publish an ecosystem method-set reference for that Gem chain; it is not an implementation TODO. Bitcoin is `🏗️` because WalletConnect does publish a `bip122` method set and Core does not implement it.

### Supported method sets

| Method set | Namespace | Core-supported methods | Published methods missing in Core | WalletConnect reference |
| --- | --- | --- | --- | --- |
| Bitcoin (TODO) | `bip122` | None | `getAccountAddresses`<br>`signMessage`<br>`signPsbt`<br>`sendTransfer` | <sub>[spec](https://docs.walletconnect.network/wallet-sdk/chain-support/bitcoin)</sub> |
| [EVM](../crates/gem_wallet_connect/src/request_handler/ethereum.rs) | `eip155` | `eth_chainId`<br>`personal_sign`<br>`eth_signTypedData`<br>`eth_signTypedData_v4`<br>`eth_signTransaction`<br>`eth_sendTransaction`<br>`wallet_addEthereumChain`<br>`wallet_switchEthereumChain` | `eth_sign`<br>`eth_sendRawTransaction` | <sub>[spec](https://docs.walletconnect.network/wallet-sdk/chain-support/evm)</sub> |
| [Solana](../crates/gem_wallet_connect/src/request_handler/solana.rs) | `solana` | `solana_signMessage`<br>`solana_signTransaction`<br>`solana_signAndSendTransaction`<br>`solana_signAllTransactions` | `solana_getAccounts`<br>`solana_requestAccounts` | <sub>[spec](https://docs.walletconnect.network/wallet-sdk/chain-support/solana)</sub> |
| [Sui](../crates/gem_wallet_connect/src/request_handler/sui.rs) | `sui` | `sui_getAccounts`<br>`sui_signPersonalMessage`<br>`sui_signTransaction`<br>`sui_signAndExecuteTransaction` | ➖ | <sub>[spec](https://docs.walletconnect.network/wallet-sdk/chain-support/sui)</sub> |
| [TON](../crates/gem_wallet_connect/src/request_handler/ton.rs) | `ton` | `ton_sendMessage`<br>`ton_signData` | ➖ | <sub>[spec](https://docs.walletconnect.network/wallet-sdk/chain-support/ton)</sub> |
| [Tron](../crates/gem_wallet_connect/src/request_handler/tron.rs) | `tron` | `tron_signMessage`<br>`tron_signTransaction`<br>`tron_sendTransaction` | `tron_getBalance` (optional) | <sub>[spec](https://docs.walletconnect.network/wallet-sdk/chain-support/tron)</sub> |

Android's EVM namespace and iOS's all-method list advertise `eth_sendRawTransaction`, but Core explicitly rejects it. The TODO table tracks this cross-platform mismatch separately from methods that are simply absent.

### Chain coverage

| Chain | Status | Namespace | Supported methods |
| --- | :---: | --- | --- |
| Bitcoin | 🏗️ | `bip122` | ❌ (0/4 published methods) |
| Bitcoin Cash | ➖ | ➖ | ➖ |
| Litecoin | ➖ | ➖ | ➖ |
| Ethereum | ✅ | `eip155` | [EVM (8)](../crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| BNB Smart Chain | ✅ | `eip155` | [EVM (8)](../crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Solana | ✅ | `solana` | [Solana (4)](../crates/gem_wallet_connect/src/request_handler/solana.rs) |
| Polygon | ✅ | `eip155` | [EVM (8)](../crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| THORChain | ➖ | ➖ | ➖ |
| MayaChain | ➖ | ➖ | ➖ |
| Cosmos | ➖ | ➖ | ➖ |
| Osmosis | ➖ | ➖ | ➖ |
| Arbitrum | ✅ | `eip155` | [EVM (8)](../crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| TON | ✅ | `ton` | [TON (2)](../crates/gem_wallet_connect/src/request_handler/ton.rs) |
| Tron | ✅ | `tron` | [Tron (3)](../crates/gem_wallet_connect/src/request_handler/tron.rs) |
| Dogecoin | ➖ | ➖ | ➖ |
| Zcash | ➖ | ➖ | ➖ |
| Optimism | ✅ | `eip155` | [EVM (8)](../crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Aptos | ➖ | ➖ | ➖ |
| Base | ✅ | `eip155` | [EVM (8)](../crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Avalanche C-Chain | ✅ | `eip155` | [EVM (8)](../crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Sui | ✅ | `sui` | [Sui (4)](../crates/gem_wallet_connect/src/request_handler/sui.rs) |
| XRP Ledger | ➖ | ➖ | ➖ |
| opBNB | ✅ | `eip155` | [EVM (8)](../crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Fantom | ✅ | `eip155` | [EVM (8)](../crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Gnosis | ✅ | `eip155` | [EVM (8)](../crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Celestia | ➖ | ➖ | ➖ |
| Injective | ➖ | ➖ | ➖ |
| Sei | ➖ | ➖ | ➖ |
| Sei EVM | ✅ | `eip155` | [EVM (8)](../crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Manta | ✅ | `eip155` | [EVM (8)](../crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Blast | ✅ | `eip155` | [EVM (8)](../crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Noble | ➖ | ➖ | ➖ |
| ZKsync | ✅ | `eip155` | [EVM (8)](../crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Linea | ✅ | `eip155` | [EVM (8)](../crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Mantle | ✅ | `eip155` | [EVM (8)](../crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Celo | ✅ | `eip155` | [EVM (8)](../crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| NEAR | ➖ | ➖ | ➖ |
| World Chain | ✅ | `eip155` | [EVM (8)](../crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Stellar | ➖ | ➖ | ➖ |
| Sonic | ✅ | `eip155` | [EVM (8)](../crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Algorand | ➖ | ➖ | ➖ |
| Polkadot | ➖ | ➖ | ➖ |
| Plasma | ✅ | `eip155` | [EVM (8)](../crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Cardano | ➖ | ➖ | ➖ |
| Abstract | ✅ | `eip155` | [EVM (8)](../crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Berachain | ✅ | `eip155` | [EVM (8)](../crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Ink | ✅ | `eip155` | [EVM (8)](../crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Unichain | ✅ | `eip155` | [EVM (8)](../crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Hyperliquid | ✅ | `eip155` | [EVM (8)](../crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| HyperCore | ➖ | ➖ | ➖ |
| Monad | ✅ | `eip155` | [EVM (8)](../crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| X Layer | ✅ | `eip155` | [EVM (8)](../crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Robinhood Chain | ✅ | `eip155` | [EVM (8)](../crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Stable | ✅ | `eip155` | [EVM (8)](../crates/gem_wallet_connect/src/request_handler/ethereum.rs) |

<sub>Reviewed 2026-07-21. Sources: [advertised chains](../gemstone/src/config/wallet_connect.rs), [method identifiers](../crates/primitives/src/wallet_connector.rs), [Core dispatcher](../crates/gem_wallet_connect/src/request_handler/mod.rs), [Android namespaces](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/bridge/Namespace.kt), and [iOS method advertisement](../../ios/Features/WalletConnector/Sources/WalletConnector/Services/WalletConnectorSigner.swift).</sub>

## Transaction-indexing providers

The EVM route configured by [`settings_chain`](../crates/settings_chain/src/lib.rs) prefers [Ankr](../crates/gem_evm/src/rpc/ankr/client.rs) for 11 mapped chains and otherwise calls Alchemy's [`alchemy_getAssetTransfers`](../crates/gem_evm/src/rpc/alchemy/client.rs). A generic Alchemy Chain API endpoint is not evidence that the Transfers API works there, so those two facts are kept separate below. The [gemstone chain factory](../gemstone/src/gateway/chain_factory.rs) currently constructs EVM clients without an indexer; its EVM address-history calls therefore return an empty result.

`➖` in the Third-party provider column means that Core uses a chain-specific implementation instead. It is not a missing capability.

| Gem chain | Core transaction-history route | Third-party provider | Verification |
| --- | --- | :---: | --- |
| Bitcoin | [Bitcoin chain client](../crates/gem_bitcoin/src/rpc/client.rs) | ➖ | ✅ Core implementation |
| Bitcoin Cash | [Bitcoin chain client](../crates/gem_bitcoin/src/rpc/client.rs) | ➖ | ✅ Core implementation |
| Litecoin | [Bitcoin chain client](../crates/gem_bitcoin/src/rpc/client.rs) | ➖ | ✅ Core implementation |
| Ethereum | [Ankr Advanced API](../crates/gem_evm/src/rpc/ankr/client.rs) | ✅ Ankr | ✅ Core mapping and exact Ankr methods documented |
| BNB Smart Chain | [Ankr Advanced API](../crates/gem_evm/src/rpc/ankr/client.rs) | ✅ Ankr | ✅ Core mapping and exact Ankr methods documented |
| Solana | [Alchemy `getTransactionsForAddress`](../crates/gem_solana/src/rpc/indexer/mod.rs) | ✅ Alchemy | ✅ Core implementation and method documented |
| Polygon | [Ankr Advanced API](../crates/gem_evm/src/rpc/ankr/client.rs) | ✅ Ankr | ✅ Core mapping and exact Ankr methods documented |
| THORChain | [Cosmos chain client](../crates/gem_cosmos/src/rpc/client.rs) | ➖ | ✅ Core implementation |
| MayaChain | [Cosmos chain client](../crates/gem_cosmos/src/rpc/client.rs) | ➖ | ✅ Core implementation |
| Cosmos | [Cosmos chain client](../crates/gem_cosmos/src/rpc/client.rs) | ➖ | ✅ Core implementation |
| Osmosis | [Cosmos chain client](../crates/gem_cosmos/src/rpc/client.rs) | ➖ | ✅ Core implementation |
| Arbitrum | [Ankr Advanced API](../crates/gem_evm/src/rpc/ankr/client.rs) | ✅ Ankr | ✅ Core mapping and exact Ankr methods documented |
| TON | [TON chain client](../crates/gem_ton/src/rpc/client.rs) | ➖ | ✅ Core implementation |
| Tron | [TronGrid](../crates/gem_tron/src/rpc/trongrid/client.rs) | ➖ | ✅ Core implementation |
| Dogecoin | [Bitcoin chain client](../crates/gem_bitcoin/src/rpc/client.rs) | ➖ | ✅ Core implementation |
| Zcash | [Bitcoin chain client](../crates/gem_bitcoin/src/rpc/client.rs) | ➖ | ✅ Core implementation |
| Optimism | [Ankr Advanced API](../crates/gem_evm/src/rpc/ankr/client.rs) | ✅ Ankr | ✅ Core mapping and exact Ankr methods documented |
| Aptos | [Aptos chain client](../crates/gem_aptos/src/rpc/client.rs) | ➖ | ✅ Core implementation |
| Base | [Ankr Advanced API](../crates/gem_evm/src/rpc/ankr/client.rs) | ✅ Ankr | ✅ Core mapping and exact Ankr methods documented |
| Avalanche C-Chain | [Ankr Advanced API](../crates/gem_evm/src/rpc/ankr/client.rs) | ✅ Ankr | ✅ Core mapping and exact Ankr methods documented |
| Sui | [Sui GraphQL](../crates/gem_sui/src/rpc/indexer/mod.rs) | ➖ | ✅ Core implementation |
| XRP Ledger | [XRP chain client](../crates/gem_xrp/src/rpc/client.rs) | ➖ | ✅ Core implementation |
| opBNB | [Alchemy Transfers API](../crates/gem_evm/src/rpc/alchemy/client.rs) | ✅ Alchemy | ✅ Core route configured; 🏗️ verify exact method support |
| Fantom | [Ankr Advanced API](../crates/gem_evm/src/rpc/ankr/client.rs) | ✅ Ankr | ✅ Core mapping and exact Ankr methods documented |
| Gnosis | [Ankr Advanced API](../crates/gem_evm/src/rpc/ankr/client.rs) | ✅ Ankr | ✅ Core mapping and exact Ankr methods documented |
| Celestia | [Cosmos chain client](../crates/gem_cosmos/src/rpc/client.rs) | ➖ | ✅ Core implementation |
| Injective | [Cosmos chain client](../crates/gem_cosmos/src/rpc/client.rs) | ➖ | ✅ Core implementation |
| Sei | [Cosmos chain client](../crates/gem_cosmos/src/rpc/client.rs) | ➖ | ✅ Core implementation |
| Sei EVM | [Alchemy Transfers API](../crates/gem_evm/src/rpc/alchemy/client.rs) | ✅ Alchemy | ✅ Core route configured; 🏗️ verify exact method support |
| Manta | [Alchemy Transfers API](../crates/gem_evm/src/rpc/alchemy/client.rs) | 🏗️ Alchemy | 🏗️ Configured hostname is absent from Alchemy's chain list and does not resolve |
| Blast | [Alchemy Transfers API](../crates/gem_evm/src/rpc/alchemy/client.rs) | ✅ Alchemy | ✅ Core route configured; 🏗️ verify exact method support |
| Noble | [Cosmos chain client](../crates/gem_cosmos/src/rpc/client.rs) | ➖ | ✅ Core implementation |
| ZKsync | [Alchemy Transfers API](../crates/gem_evm/src/rpc/alchemy/client.rs) | ✅ Alchemy | ✅ Core route configured; 🏗️ verify exact method support |
| Linea | [Ankr Advanced API](../crates/gem_evm/src/rpc/ankr/client.rs) | ✅ Ankr | ✅ Core mapping and exact Ankr methods documented |
| Mantle | [Alchemy Transfers API](../crates/gem_evm/src/rpc/alchemy/client.rs) | ✅ Alchemy | ✅ Core route configured; 🏗️ verify exact method support |
| Celo | [Alchemy Transfers API](../crates/gem_evm/src/rpc/alchemy/client.rs) | ✅ Alchemy | ✅ Core route configured; 🏗️ verify exact method support |
| NEAR | [FastNear Transfers and Transactions APIs](../crates/gem_near/src/rpc/indexer/mod.rs) | ✅ FastNear | ✅ Full indexed transfers with transaction fees |
| World Chain | [Alchemy Transfers API](../crates/gem_evm/src/rpc/alchemy/client.rs) | ✅ Alchemy | ✅ Core route configured; 🏗️ verify exact method support |
| Stellar | [Stellar chain client](../crates/gem_stellar/src/rpc/client.rs) | ➖ | ✅ Core implementation |
| Sonic | [Alchemy Transfers API](../crates/gem_evm/src/rpc/alchemy/client.rs) | ✅ Alchemy | ✅ Core route configured; 🏗️ verify exact method support |
| Algorand | [Algorand Indexer](../crates/gem_algorand/src/rpc/indexer/mod.rs) | ➖ | ✅ Core implementation |
| Polkadot | [Subscan Asset Hub API](../crates/gem_polkadot/src/rpc/indexer/mod.rs) | ✅ Subscan | ✅ Full transfer records; authenticated API key required |
| Plasma | [Alchemy Transfers API](../crates/gem_evm/src/rpc/alchemy/client.rs) | ✅ Alchemy | ✅ Core route configured; 🏗️ verify exact method support |
| Cardano | [Cardano chain client](../crates/gem_cardano/src/rpc/client.rs) | ➖ | ✅ Core implementation |
| Abstract | [Alchemy Transfers API](../crates/gem_evm/src/rpc/alchemy/client.rs) | ✅ Alchemy | ✅ Core route configured; 🏗️ verify exact method support |
| Berachain | [Alchemy Transfers API](../crates/gem_evm/src/rpc/alchemy/client.rs) | ✅ Alchemy | ✅ Core route configured; 🏗️ verify exact method support |
| Ink | [Alchemy Transfers API](../crates/gem_evm/src/rpc/alchemy/client.rs) | ✅ Alchemy | ✅ Core route configured; 🏗️ verify exact method support |
| Unichain | [Alchemy Transfers API](../crates/gem_evm/src/rpc/alchemy/client.rs) | ✅ Alchemy | ✅ Core route configured; 🏗️ verify exact method support |
| Hyperliquid | [Alchemy Transfers API](../crates/gem_evm/src/rpc/alchemy/client.rs) | ✅ Alchemy | ✅ Core route configured for HyperEVM; 🏗️ verify exact method support |
| HyperCore | [HyperCore chain client](../crates/gem_hypercore/src/rpc/client.rs) | ➖ | ✅ Core implementation |
| Monad | [Alchemy Transfers API](../crates/gem_evm/src/rpc/alchemy/client.rs) | ✅ Alchemy | ✅ Core route configured; 🏗️ verify exact method support |
| X Layer | [Ankr Advanced API](../crates/gem_evm/src/rpc/ankr/client.rs) | ✅ Ankr | ✅ Core mapping and exact Ankr methods documented |
| Robinhood Chain | [Alchemy Transfers API](../crates/gem_evm/src/rpc/alchemy/client.rs) | ✅ Alchemy | ✅ Alchemy endpoint and Core route exist; 🏗️ verify exact method support |
| Stable | [Alchemy Transfers API](../crates/gem_evm/src/rpc/alchemy/client.rs) | ✅ Alchemy | ✅ Core route configured; 🏗️ verify exact method support |

[Ankr's Advanced API](https://www.ankr.com/docs/advanced-api/overview/) also advertises Flare, Scroll, Story, Syscoin, Taiko, Telos, and Xai, which are not Gem chains. Alchemy's generic [Chain API list](https://www.alchemy.com/docs/reference/node-supported-chains) is retained as endpoint evidence only; method-level support must be checked against the [Transfers API](https://www.alchemy.com/docs/reference/transfers-api-quickstart) or an authenticated request.

<sub>Reviewed 2026-07-21. External method references: [Ankr Advanced API](https://www.ankr.com/docs/advanced-api/overview/) · [Alchemy Transfers API](https://www.alchemy.com/docs/reference/transfers-api-quickstart) · [Alchemy Solana history method](https://www.alchemy.com/docs/chains/solana/solana-api-endpoints/get-transactions-for-address) · [Sui GraphQL](https://docs.sui.io/concepts/data-access/graphql-rpc) · [TronGrid account transactions](https://developers.tron.network/reference/get-transaction-info-by-account-address) · [Algorand Indexer](https://dev.algorand.co/reference/rest-api/indexer/).</sub>

## Swap providers

These tables compare Core route eligibility with current provider-advertised support. A Core chain means that the provider's `supported_assets()` exposes at least one asset on that chain; it does not mean that every token or chain pair is available. `Coverage difference` only mentions chains in Gem's 54-chain [`Chain`](../crates/primitives/src/chain.rs) enum.

### On-chain providers

| Provider | Core coverage | Core chains | Coverage difference | References |
| --- | :---: | --- | --- | --- |
| [Uniswap v3](../crates/swapper/src/uniswap/v3/provider.rs) | 16/54 | Ethereum, Optimism, Arbitrum, Polygon, Avalanche C-Chain, Base, BNB Smart Chain, ZKsync, Celo, Blast, World Chain, Unichain, Monad, X Layer, Stable, Robinhood Chain | Core-only relative to the current official list: Blast, Stable | <sub>[code](../crates/gem_evm/src/uniswap/deployment/v3.rs) · [current deployments](https://developers.uniswap.org/docs/protocols/v3/deployments) · [Blast sunset](https://developers.uniswap.org/docs/changelog/active-notifications/sunset-of-blast-support) · [Stable deployment](https://swap.stable.xyz/deployments)</sub> |
| [Uniswap v4](../crates/swapper/src/uniswap/v4/provider.rs) | 16/54 | Ethereum, Optimism, Arbitrum, Polygon, Avalanche C-Chain, Base, BNB Smart Chain, Blast, Linea, World Chain, Unichain, Celo, Monad, Ink, X Layer, Robinhood Chain | ➖ | <sub>[code](../crates/gem_evm/src/uniswap/deployment/v4.rs) · [deployments](https://github.com/Uniswap/contracts/blob/main/deployments/index.md)</sub> |
| [PancakeSwap v3](../crates/swapper/src/uniswap/v3/provider.rs) | 8/54 | BNB Smart Chain, opBNB, Arbitrum, Linea, Base, ZKsync, Monad, Robinhood Chain | Missing: Ethereum | <sub>[code](../crates/gem_evm/src/uniswap/deployment/v3.rs) · [v3 router deployments](https://developer.pancakeswap.finance/contracts/universal-router/addresses)</sub> |
| [OKX DEX](../crates/swapper/src/okx/provider.rs) | 22/54 | Solana, Tron, Ethereum, BNB Smart Chain, Polygon, Arbitrum, Optimism, Base, Avalanche C-Chain, Fantom, Manta, Blast, ZKsync, Linea, Mantle, Plasma, Hyperliquid, Sonic, Unichain, Monad, X Layer, Robinhood Chain | Missing: Sui, TON | <sub>[code](../crates/swapper/src/okx/provider.rs) · [chains API](https://web3.okx.com/onchainos/dev-docs-v5/dex-api/dex-get-aggregator-supported-chains)</sub> |
| [Oku](../crates/swapper/src/uniswap/v3/provider.rs) | 5/54 | Sonic, Mantle, Gnosis, Plasma, Sei EVM | Not assigned to Oku in Core: Ethereum, Optimism, Arbitrum, Polygon, ZKsync, Base, BNB Smart Chain, Avalanche C-Chain, Blast, Linea, Celo, Manta, Unichain, World Chain, Monad | <sub>[code](../crates/gem_evm/src/uniswap/deployment/v3.rs) · [deployments](https://docs.oku.trade/home/extra-information/deployed-contracts)</sub> |
| [Wagmi](../crates/swapper/src/uniswap/v3/provider.rs) | 1/54 | Sonic | Missing: Ethereum, BNB Smart Chain, Avalanche C-Chain, Polygon, Fantom, Arbitrum, Optimism, Base | <sub>[code](../crates/gem_evm/src/uniswap/deployment/v3.rs) · [contracts](https://docs.wagmi.com/wagmi/contracts)</sub> |
| [Aerodrome](../crates/swapper/src/uniswap/v3/provider.rs) | 1/54 | Base | ➖ | <sub>[code](../crates/gem_evm/src/uniswap/deployment/v3.rs) · [contracts](https://aerodrome.finance/security)</sub> |
| [Jupiter](../crates/swapper/src/jupiter/provider.rs) | 1/54 | Solana | ➖ | <sub>[code](../crates/swapper/src/jupiter/provider.rs) · [quote API](https://dev.jup.ag/docs/swap/v1/get-quote)</sub> |
| [Panora](../crates/swapper/src/panora/provider.rs) | 1/54 | Aptos | ➖ | <sub>[code](../crates/swapper/src/panora/provider.rs) · [swap API](https://docs.panora.exchange/developer/swap/api)</sub> |
| [STON.fi v2](../crates/swapper/src/stonfi/provider.rs) | 1/54 | TON | ➖ | <sub>[code](../crates/swapper/src/stonfi/provider.rs) · [DEX API](https://docs.ston.fi/developer-section/dex/api/reference)</sub> |
| [Cetus CLMM](../crates/swapper/src/cetus_clmm/provider.rs) | 1/54 | Sui | Missing: Aptos | <sub>[code](../crates/swapper/src/cetus_clmm/provider.rs) · [Sui developer docs](https://cetus-1.gitbook.io/cetus-developer-docs) · [Cetus chain support](https://cetus-1.gitbook.io/cetus-docs/guides/faq)</sub> |

#### Behavior

| Provider | Amount | Slippage |
| --- | --- | --- |
| [Uniswap v3](../crates/swapper/src/uniswap/v3/provider.rs) | Fixed | Exact |
| [Uniswap v4](../crates/swapper/src/uniswap/v4/provider.rs) | Fixed | Exact |
| [PancakeSwap v3](../crates/swapper/src/uniswap/v3/provider.rs) | Fixed | Exact |
| [OKX DEX](../crates/swapper/src/okx/provider.rs) | Flexible | Auto |
| [Oku](../crates/swapper/src/uniswap/v3/provider.rs) | Fixed | Exact |
| [Wagmi](../crates/swapper/src/uniswap/v3/provider.rs) | Fixed | Exact |
| [Aerodrome](../crates/swapper/src/uniswap/v3/provider.rs) | Fixed | Exact |
| [Jupiter](../crates/swapper/src/jupiter/provider.rs) | Fixed | Exact |
| [Panora](../crates/swapper/src/panora/provider.rs) | Fixed | Exact |
| [STON.fi v2](../crates/swapper/src/stonfi/provider.rs) | Fixed | Exact |
| [Cetus CLMM](../crates/swapper/src/cetus_clmm/provider.rs) | Fixed | Exact |

Provider-specific status tracking and vault discovery are `➖` for on-chain routes; transaction tracking belongs to the chain client.

<sub>Sources: [provider modes and slippage](../crates/swapper/src/models.rs) and [amount-mode implementations](../crates/swapper/src).</sub>

<sub>Additional OKX gap evidence: [Sui guide](https://web3.okx.com/onchainos/dev-docs-v5/dex-api/dex-use-swap-sui-quick-start) and [TON guide](https://web3.okx.com/build/docs/waas/dex-use-swap-ton-quick-start).</sub>

### Cross-chain and bridge providers

| Provider | Mode | Core coverage | Core chains | Coverage difference | References |
| --- | --- | :---: | --- | --- | --- |
| [NEAR Intents](../crates/swapper/src/near_intents/provider.rs) | Omnichain | 27/54 | NEAR, Ethereum, Bitcoin, Solana, Sui, Arbitrum, Base, Optimism, Avalanche C-Chain, BNB Smart Chain, Polygon, TON, Tron, Dogecoin, XRP Ledger, Cardano, Berachain, Aptos, Zcash, Gnosis, Stellar, Litecoin, Bitcoin Cash, Monad, X Layer, Plasma, Abstract | Missing: HyperCore | <sub>[code](../crates/swapper/src/near_intents/assets.rs) · [live token API](https://1click.chaindefuser.com/v0/tokens) · [chain support](https://docs.near-intents.org/resources/chain-support)</sub> |
| [Relay](../crates/swapper/src/relay/provider.rs) | Omnichain | 18/54 | Ethereum, BNB Smart Chain, Base, Arbitrum, Optimism, Polygon, Avalanche C-Chain, Linea, ZKsync, Hyperliquid, Sei EVM, Berachain, Manta, Sonic, Abstract, Celo, Stable, Robinhood Chain | Missing: Bitcoin, Solana, TON, Tron, Gnosis, Mantle, Blast, World Chain, Ink, Unichain, Monad, Plasma, HyperCore | <sub>[code](../crates/swapper/src/relay/asset.rs) · [live chains API](https://api.relay.link/chains)</sub> |
| [Across](../crates/swapper/src/across/provider.rs) | Bridge | 16/54 | Ethereum, Optimism, Polygon, Arbitrum, Base, Hyperliquid, Linea, ZKsync, World Chain, Ink, Unichain, Monad, BNB Smart Chain, Plasma, Robinhood Chain, Tron | Missing: Avalanche C-Chain, Solana, Blast, HyperCore | <sub>[code](../crates/gem_evm/src/across/deployment.rs) · [live chains API](https://app.across.to/api/swap/chains)</sub> |
| [THORChain](../crates/swapper/src/thorchain/provider.rs) | Omnichain | 14/54 | Dogecoin, THORChain, Ethereum, Cosmos, Bitcoin, Bitcoin Cash, Litecoin, BNB Smart Chain, Avalanche C-Chain, Base, XRP Ledger, Tron, Solana, Zcash | Core-only: Zcash | <sub>[code](../crates/swapper/src/thorchain/chain.rs) · [supported chains](https://dev.thorchain.org/concepts/querying-thorchain.html) · [live inbound-address API](https://gateway.liquify.com/chain/thorchain_api/thorchain/inbound_addresses)</sub> |
| [Mayan](../crates/swapper/src/mayan/provider.rs) | Cross-chain | 14/54 | Ethereum, Solana, Sui, BNB Smart Chain, Base, Polygon, Avalanche C-Chain, Arbitrum, Optimism, Linea, Unichain, Monad, Hyperliquid, HyperCore | ➖ | <sub>[code](../crates/swapper/src/mayan/asset.rs) · [quote API](https://docs.mayan.finance/integration/quote-api) · [live configuration](https://sia.mayan.finance/v10/init)</sub> |
| [Squid](../crates/swapper/src/squid/provider.rs) | Cross-chain | 6/54 | Cosmos, Osmosis, Celestia, Injective, Sei, Noble | Missing: Ethereum, BNB Smart Chain, Arbitrum, Optimism, Polygon, Avalanche C-Chain, Base, Fantom, Linea, Mantle, Celo, Blast, Berachain, Gnosis, Sonic, Hyperliquid, Bitcoin, Solana, Sui, XRP Ledger, Stellar | <sub>[code](../crates/swapper/src/squid/provider.rs) · [supported chains](https://docs.squidrouter.com/api-and-sdk-integration/key-concepts/get-supported-tokens-and-chains)</sub> |
| [MayaChain](../crates/swapper/src/thorchain/provider.rs) | Cross-chain | 6/54 | THORChain, Bitcoin, Ethereum, Arbitrum, Zcash, Cardano | ➖ | <sub>[code](../crates/swapper/src/thorchain/chain.rs) · [live chains API](https://mayanode.mayachain.info/mayachain/inbound_addresses)</sub> |
| [Chainflip](../crates/swapper/src/chainflip/provider.rs) | Cross-chain | 5/54 | Bitcoin (destination only), Ethereum, Solana, Tron, Arbitrum | Missing: Polkadot | <sub>[asset mapping](../crates/swapper/src/chainflip/client/model.rs) · [source-chain guard](../crates/swapper/src/chainflip/provider.rs) · [supported chains](https://docs.chainflip.io/protocol/supported-chains-assets/chains-assets)</sub> |
| [Hyperliquid](../crates/swapper/src/hyperliquid/provider/hyperliquid.rs) | Omnichain | 2/54 | HyperCore, Hyperliquid | ➖ | <sub>[code](../crates/swapper/src/hyperliquid/provider/hyperliquid.rs) · [Core ↔ EVM transfers](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/hyperevm/hypercore-less-than-greater-than-hyperevm-transfers)</sub> |

#### Behavior

| Provider | Amount | Slippage | Status tracking | Vault discovery |
| --- | --- | --- | :---: | :---: |
| [NEAR Intents](../crates/swapper/src/near_intents/provider.rs) | Flexible | Exact | ✅ | ✅ |
| [Relay](../crates/swapper/src/relay/provider.rs) | Fixed | Exact | ✅ | ✅ |
| [Across](../crates/swapper/src/across/provider.rs) | Fixed | Exact | ✅ | ✅ |
| [THORChain](../crates/swapper/src/thorchain/provider.rs) | Fixed from EVM; flexible otherwise | Exact | ✅ | ✅ |
| [Mayan](../crates/swapper/src/mayan/provider.rs) | Fixed | Auto | ✅ | ✅ |
| [Squid](../crates/swapper/src/squid/provider.rs) | Fixed | Auto | ✅ | ✅ |
| [MayaChain](../crates/swapper/src/thorchain/provider.rs) | Fixed from EVM; flexible otherwise | Exact | ✅ | ✅ |
| [Chainflip](../crates/swapper/src/chainflip/provider.rs) | Fixed | Exact | ✅ | ✅ |
| [Hyperliquid](../crates/swapper/src/hyperliquid/provider/hyperliquid.rs) | Flexible | Exact | ➖ (assumed complete) | ➖ |

<sub>Sources: [provider modes and slippage](../crates/swapper/src/models.rs), [amount-mode implementations](../crates/swapper/src), and [active provider registry](../crates/swapper/src/swapper.rs).</sub>

`Omnichain` providers may be eligible for selected same-chain routes as well as cross-chain routes. `Bridge` and `Cross-chain` providers require different source and destination chains.

<sub>Reviewed 2026-07-21. Active-provider source of truth: [`GemSwapper::new`](../crates/swapper/src/swapper.rs). `CetusAggregator` and `Orca` remain inactive [`SwapProvider`](../crates/primitives/src/swap_provider.rs) variants.</sub>

## Fiat providers

The active registry constructs all six providers below. `Order lookup` means Core can poll a provider order directly; webhook processing is supported independently for every active provider.

| Provider | Buy quotes | Sell quotes | Asset catalog | Countries | Checkout | Webhooks | Order lookup |
| --- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| [MoonPay](../crates/fiat/src/providers/moonpay/provider.rs) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| [Mercuryo](../crates/fiat/src/providers/mercuryo/provider.rs) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| [Transak](../crates/fiat/src/providers/transak/provider.rs) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| [Banxa](../crates/fiat/src/providers/banxa/provider.rs) | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ |
| [Paybis](../crates/fiat/src/providers/paybis/provider.rs) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| [Cash App / Flashnet](../crates/fiat/src/providers/flashnet/provider.rs) | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ |

Card, Apple Pay, and Google Pay are the default payment methods. Flashnet overrides that list with Cash App. Country data is provider-backed except Paybis, which uses Core's country-status mapping, and Flashnet, which is US-only.

<sub>Reviewed 2026-07-21. Sources: [active provider factory](../crates/fiat/src/lib.rs), [provider contract](../crates/fiat/src/provider.rs), and [Paybis payment methods](../crates/fiat/src/providers/paybis/mapper.rs).</sub>

## NFT providers

The NFT registry selects providers by chain and falls through in registration order when a provider request fails. Wallet assets, collection metadata, and individual asset metadata are part of the shared provider contract.

| Provider | Active | Chains | Wallet assets | Collections | Asset details | Notes |
| --- | :---: | --- | :---: | :---: | :---: | --- |
| [OpenSea](../crates/nft/src/providers/opensea/provider.rs) | ✅ | Ethereum, Polygon | ✅ | ✅ | ✅ | First registered provider |
| [Magic Eden Solana](../crates/nft/src/providers/magiceden/solana/provider.rs) | ✅ | Solana | ✅ | ✅ | ✅ | Dedicated Solana API |
| [Magic Eden EVM](../crates/nft/src/providers/magiceden/evm/provider.rs) | ✅ | BNB Smart Chain | ✅ | ✅ | ✅ | EVM adapter currently exposes BNB Smart Chain only |
| [TON](../crates/nft/src/providers/ton/provider.rs) | ✅ | TON | ✅ | ✅ | ✅ | Uses indexed on-chain data with off-chain metadata fallback |

<sub>Reviewed 2026-07-21. Sources: [active provider factory](../crates/nft/src/factory.rs), [provider contract and fallback behavior](../crates/nft/src/provider.rs), [supported NFT chains](../crates/primitives/src/chain_nft.rs), and [NFT settings](../crates/settings/src/lib.rs).</sub>

## Actionable TODOs

This backlog only includes gaps that apply to an existing Gem chain, an active provider, or a current cross-platform inconsistency. Non-applicable and speculative capabilities—such as Bitcoin simulation or Bitcoin NFTs—and provider chains that Gem does not support are intentionally excluded.

| Area | Work | Applicable scope | References | Status |
| --- | --- | --- | --- | :---: |
| Transaction indexing | Inject an EVM indexer into gemstone or explicitly remove EVM address history from that API surface | All EVM chains created by gemstone | <sub>[factory](../gemstone/src/gateway/chain_factory.rs) · [unsupported default](../crates/gem_evm/src/rpc/client.rs) · [configured backend path](../crates/settings_chain/src/lib.rs)</sub> | 🏗️ |
| Transaction indexing | Verify Manta against a working Alchemy endpoint; keep the route only if it is still supported | Manta | <sub>[Core endpoint](../crates/gem_evm/src/rpc/alchemy/client.rs) · [Alchemy chains](https://www.alchemy.com/docs/reference/node-supported-chains)</sub> | 🏗️ |
| WalletConnect | Align EVM method policy: remove `eth_sendRawTransaction` from platform advertisements or implement it in Core, and explicitly decide whether to support `eth_sign` | EVM chains | <sub>[Core handler](../crates/gem_wallet_connect/src/request_handler/ethereum.rs) · [EVM method reference](https://docs.walletconnect.network/wallet-sdk/chain-support/evm) · [Android](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/bridge/Namespace.kt) · [iOS](../../ios/Features/WalletConnector/Sources/WalletConnector/Services/WalletConnectorSigner.swift)</sub> | 🏗️ |
| WalletConnect | Add the published account-discovery methods | Solana | <sub>[Core handler](../crates/gem_wallet_connect/src/request_handler/solana.rs) · [Solana method reference](https://docs.walletconnect.network/wallet-sdk/chain-support/solana)</sub> | 🏗️ |
| WalletConnect | Decide whether to implement the optional published balance method | Tron | <sub>[Core handler](../crates/gem_wallet_connect/src/request_handler/tron.rs) · [Tron method reference](https://docs.walletconnect.network/wallet-sdk/chain-support/tron)</sub> | 🏗️ |
| WalletConnect | Add the published Bitcoin namespace and request handler | Bitcoin | <sub>[Core dispatcher](../crates/gem_wallet_connect/src/request_handler/mod.rs) · [WalletConnect Bitcoin spec](https://docs.walletconnect.network/wallet-sdk/chain-support/bitcoin)</sub> | 🏗️ |
| Uniswap v3 | Confirm whether the direct Blast contract route should remain after Uniswap removed Blast from its supported references | Blast | <sub>[Core deployment](../crates/gem_evm/src/uniswap/deployment/v3.rs) · [sunset notice](https://developers.uniswap.org/docs/changelog/active-notifications/sunset-of-blast-support)</sub> | 🏗️ |
| PancakeSwap v3 | Add the missing v3 router deployment | Ethereum | <sub>[Core deployment](../crates/gem_evm/src/uniswap/deployment/v3.rs) · [provider deployment](https://developer.pancakeswap.finance/contracts/universal-router/addresses)</sub> | 🏗️ |
| Oku | Add compatible provider deployment mappings not already assigned to the Oku adapter | Ethereum, Optimism, Arbitrum, Polygon, ZKsync, Base, BNB Smart Chain, Avalanche C-Chain, Blast, Linea, Celo, Manta, Unichain, World Chain, Monad | <sub>[Core deployment](../crates/gem_evm/src/uniswap/deployment/v3.rs) · [provider deployments](https://docs.oku.trade/home/extra-information/deployed-contracts)</sub> | 🏗️ |
| Wagmi | Add the remaining documented provider deployments | Ethereum, BNB Smart Chain, Avalanche C-Chain, Polygon, Fantom, Arbitrum, Optimism, Base | <sub>[Core deployment](../crates/gem_evm/src/uniswap/deployment/v3.rs) · [provider contracts](https://docs.wagmi.com/wagmi/contracts)</sub> | 🏗️ |
| Cetus CLMM | Add an Aptos-specific quote and transaction implementation | Aptos | <sub>[Core Sui provider](../crates/swapper/src/cetus_clmm/provider.rs) · [Cetus chain support](https://cetus-1.gitbook.io/cetus-docs/guides/faq)</sub> | 🏗️ |
| OKX DEX | Add the Sui quote and transaction path | Sui | <sub>[Core provider](../crates/swapper/src/okx/provider.rs) · [Sui guide](https://web3.okx.com/onchainos/dev-docs-v5/dex-api/dex-use-swap-sui-quick-start)</sub> | 🏗️ |
| OKX DEX | Add the TON quote and transaction path | TON | <sub>[Core provider](../crates/swapper/src/okx/provider.rs) · [TON guide](https://web3.okx.com/build/docs/waas/dex-use-swap-ton-quick-start)</sub> | 🏗️ |
| Relay | Add the missing EVM asset mappings | Gnosis, Mantle, Blast, World Chain, Ink, Unichain, Monad, Plasma | <sub>[Core mapping](../crates/swapper/src/relay/asset.rs) · [live chains API](https://api.relay.link/chains)</sub> | 🏗️ |
| Relay | Add non-EVM quote and transaction implementations before exposing the remaining provider chains | Bitcoin, Solana, TON, Tron, HyperCore | <sub>[Core EVM-only transaction model](../crates/swapper/src/relay/model.rs) · [live chains API](https://api.relay.link/chains)</sub> | 🏗️ |
| Across | Add the missing EVM routes | Avalanche C-Chain, Blast | <sub>[Core mapping](../crates/gem_evm/src/across/deployment.rs) · [live chains API](https://app.across.to/api/swap/chains)</sub> | 🏗️ |
| Across | Add non-EVM quote and transaction implementations | Solana, HyperCore | <sub>[Core provider](../crates/swapper/src/across/provider.rs) · [live chains API](https://app.across.to/api/swap/chains)</sub> | 🏗️ |
| Squid | Add an EVM quote and transaction implementation | Ethereum, BNB Smart Chain, Arbitrum, Optimism, Polygon, Avalanche C-Chain, Base, Fantom, Linea, Mantle, Celo, Blast, Berachain, Gnosis, Sonic, Hyperliquid | <sub>[Core Cosmos transaction path](../crates/swapper/src/squid/provider.rs) · [supported chains](https://docs.squidrouter.com/api-and-sdk-integration/key-concepts/get-supported-tokens-and-chains)</sub> | 🏗️ |
| Squid | Add the corresponding non-EVM quote and transaction implementations | Bitcoin, Solana, Sui, XRP Ledger, Stellar | <sub>[Core provider](../crates/swapper/src/squid/provider.rs) · [supported chains](https://docs.squidrouter.com/api-and-sdk-integration/key-concepts/get-supported-tokens-and-chains)</sub> | 🏗️ |
| Chainflip | Add the Bitcoin source transaction path; it is currently destination-only | Bitcoin | <sub>[source-chain guard](../crates/swapper/src/chainflip/provider.rs) · [Core asset mapping](../crates/swapper/src/chainflip/client/model.rs) · [provider chains](https://docs.chainflip.io/protocol/supported-chains-assets/chains-assets)</sub> | 🏗️ |
| Chainflip | Add the provider-supported relay-chain asset and transaction path | Polkadot | <sub>[Core asset mapping](../crates/swapper/src/chainflip/client/model.rs) · [provider chains](https://docs.chainflip.io/protocol/supported-chains-assets/chains-assets)</sub> | 🏗️ |
| THORChain | Remove or disable the Core-only route unless it becomes active in THORChain | Zcash | <sub>[Core mapping](../crates/swapper/src/thorchain/chain.rs) · [supported chains](https://dev.thorchain.org/concepts/querying-thorchain.html) · [live chains API](https://gateway.liquify.com/chain/thorchain_api/thorchain/inbound_addresses)</sub> | 🏗️ |
| NEAR Intents | Add the provider-advertised asset mapping | HyperCore | <sub>[Core mapping](../crates/swapper/src/near_intents/assets.rs) · [live token API](https://1click.chaindefuser.com/v0/tokens)</sub> | 🏗️ |
