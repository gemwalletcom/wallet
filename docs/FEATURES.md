# Core Features and Providers

These tables describe the current implementation, not a product promise. `Address history` means that `get_transactions_by_address` has a chain-specific implementation; it does not imply that every client constructor wires that implementation. `Simulation` means that the chain overrides the default unsupported simulation behavior. `WalletConnect` means that the chain is advertised for sessions and has request handling.

Availability legend used across comparison tables:

- ✅ = supported
- ❌ = not supported
- ➖ = not applicable
- 🏗️ = TODO
- ⚪ = not planned
- 🪦 = deprecated

Use `➖` only when the capability does not apply to the chain or provider. Use `❌` when it applies but is unavailable. Use `⚪` when support is intentionally not planned. Use `🪦` when the provider has deprecated support.

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
| Osmosis | Cosmos | ✅ | ❌ | ➖ | ✅ | ✅ | ❌ | ❌ |
| Arbitrum | EVM | ✅ | ✅ | ✅ | ✅ | ➖ | ❌ | ✅ |
| TON | TON | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ |
| Tron | Tron | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ |
| Dogecoin | Bitcoin | ✅ | ➖ | ➖ | ✅ | ➖ | ➖ | ➖ |
| Zcash | Bitcoin | ✅ | ➖ | ➖ | ✅ | ➖ | ➖ | ➖ |
| Optimism | EVM | ✅ | ✅ | ✅ | ✅ | ➖ | ❌ | ✅ |
| Aptos | Aptos | ✅ | ❌ | ➖ | ✅ | ✅ | ❌ | ❌ |
| Base | EVM | ✅ | ✅ | ✅ | ✅ | ➖ | ❌ | ✅ |
| Avalanche C-Chain | EVM | ✅ | ✅ | ✅ | ✅ | ➖ | ❌ | ✅ |
| Sui | Sui | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ |
| XRP Ledger | XRP | ✅ | ❌ | ➖ | ✅ | ➖ | ❌ | ❌ |
| opBNB | EVM | ❌ | ✅ | ✅ | ✅ | ➖ | ❌ | ❌ |
| Fantom | EVM | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ |
| Gnosis | EVM | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ |
| Celestia | Cosmos | ✅ | ❌ | ➖ | ❌ | ✅ | ➖ | ❌ |
| Injective | Cosmos | ✅ | ❌ | ➖ | ❌ | ✅ | ❌ | ❌ |
| Sei | Cosmos | ✅ | ❌ | ➖ | ❌ | ✅ | ❌ | ❌ |
| Sei EVM | EVM | ❌ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| Manta | EVM | ❌ | ✅ | ✅ | ✅ | ➖ | ❌ | ❌ |
| Blast | EVM | ✅ | ✅ | ✅ | ✅ | ➖ | ❌ | ❌ |
| Noble | Cosmos | ✅ | ❌ | ➖ | ❌ | ➖ | ➖ | ❌ |
| ZKsync | EVM | ✅ | ✅ | ✅ | ✅ | ➖ | ❌ | ✅ |
| Linea | EVM | ✅ | ✅ | ✅ | ✅ | ➖ | ❌ | ✅ |
| Mantle | EVM | ❌ | ✅ | ✅ | ✅ | ➖ | ❌ | ❌ |
| Celo | EVM | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ |
| NEAR | NEAR | ✅ | ❌ | ➖ | ✅ | ❌ | ❌ | ❌ |
| World Chain | EVM | ✅ | ✅ | ✅ | ✅ | ➖ | ❌ | ❌ |
| Stellar | Stellar | ✅ | ❌ | ➖ | ✅ | ➖ | ❌ | ❌ |
| Sonic | EVM | ❌ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| Algorand | Algorand | ✅ | ❌ | ➖ | ✅ | ❌ | ❌ | ❌ |
| Polkadot | Polkadot | ✅ | ❌ | ➖ | ❌ | ❌ | ❌ | ❌ |
| Plasma | EVM | ❌ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
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
| Stable | EVM | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ |
| Tempo | EVM | ❌ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |

<sub>Reviewed 2026-07-21. Sources: [chain list](../core/crates/primitives/src/chain.rs), [feature configuration](../core/crates/primitives/src/chain_config.rs), [address-history contract](../core/crates/chain_traits/src/lib.rs), [simulation implementations](../core/crates/gem_evm/src/provider/simulation.rs), [Solana](../core/crates/gem_solana/src/provider/simulation.rs), [Sui](../core/crates/gem_sui/src/provider/simulation.rs), [TON](../core/crates/gem_ton/src/provider/simulation.rs), [Tron](../core/crates/gem_tron/src/provider/simulation.rs), [WalletConnect chain configuration](../core/gemstone/src/config/wallet_connect.rs), and [WalletConnect request handlers](../core/crates/gem_wallet_connect/src/request_handler/mod.rs).</sub>

## WalletConnect

The status is based on both the chains returned by Core configuration and the methods accepted by the Core request dispatcher. Method-set links open the chain-specific handler.

In this section, `➖` means WalletConnect does not publish an ecosystem method-set reference for that Gem chain; it is not an implementation TODO. Bitcoin is `🏗️` because WalletConnect does publish a `bip122` method set and Core does not implement it.

### Supported method sets

| Method set | Namespace | Core-supported methods | Published methods missing in Core | WalletConnect reference |
| --- | --- | --- | --- | --- |
| Bitcoin (TODO) | `bip122` | None | `getAccountAddresses`<br>`signMessage`<br>`signPsbt`<br>`sendTransfer` | <sub>[spec](https://docs.walletconnect.network/wallet-sdk/chain-support/bitcoin)</sub> |
| [EVM](../core/crates/gem_wallet_connect/src/request_handler/ethereum.rs) | `eip155` | `eth_chainId`<br>`personal_sign`<br>`eth_signTypedData`<br>`eth_signTypedData_v4`<br>`eth_signTransaction`<br>`eth_sendTransaction`<br>`wallet_addEthereumChain`<br>`wallet_switchEthereumChain` | `eth_sign`<br>`eth_sendRawTransaction` | <sub>[spec](https://docs.walletconnect.network/wallet-sdk/chain-support/evm)</sub> |
| [Solana](../core/crates/gem_wallet_connect/src/request_handler/solana.rs) | `solana` | `solana_signMessage`<br>`solana_signTransaction`<br>`solana_signAndSendTransaction`<br>`solana_signAllTransactions` | `solana_getAccounts`<br>`solana_requestAccounts` | <sub>[spec](https://docs.walletconnect.network/wallet-sdk/chain-support/solana)</sub> |
| [Sui](../core/crates/gem_wallet_connect/src/request_handler/sui.rs) | `sui` | `sui_getAccounts`<br>`sui_signPersonalMessage`<br>`sui_signTransaction`<br>`sui_signAndExecuteTransaction` | ➖ | <sub>[spec](https://docs.walletconnect.network/wallet-sdk/chain-support/sui)</sub> |
| [TON](../core/crates/gem_wallet_connect/src/request_handler/ton.rs) | `ton` | `ton_sendMessage`<br>`ton_signData` | ➖ | <sub>[spec](https://docs.walletconnect.network/wallet-sdk/chain-support/ton)</sub> |
| [Tron](../core/crates/gem_wallet_connect/src/request_handler/tron.rs) | `tron` | `tron_signMessage`<br>`tron_signTransaction`<br>`tron_sendTransaction` | `tron_getBalance` (optional) | <sub>[spec](https://docs.walletconnect.network/wallet-sdk/chain-support/tron)</sub> |

Android's EVM namespace and iOS's all-method list advertise `eth_sendRawTransaction`, but Core explicitly rejects it. The TODO table tracks this cross-platform mismatch separately from methods that are simply absent.

### Chain coverage

| Chain | Status | Namespace | Supported methods |
| --- | :---: | --- | --- |
| Bitcoin | 🏗️ | `bip122` | ❌ (0/4 published methods) |
| Bitcoin Cash | ➖ | ➖ | ➖ |
| Litecoin | ➖ | ➖ | ➖ |
| Ethereum | ✅ | `eip155` | [EVM (8)](../core/crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| BNB Smart Chain | ✅ | `eip155` | [EVM (8)](../core/crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Solana | ✅ | `solana` | [Solana (4)](../core/crates/gem_wallet_connect/src/request_handler/solana.rs) |
| Polygon | ✅ | `eip155` | [EVM (8)](../core/crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| THORChain | ➖ | ➖ | ➖ |
| MayaChain | ➖ | ➖ | ➖ |
| Cosmos | ➖ | ➖ | ➖ |
| Osmosis | ➖ | ➖ | ➖ |
| Arbitrum | ✅ | `eip155` | [EVM (8)](../core/crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| TON | ✅ | `ton` | [TON (2)](../core/crates/gem_wallet_connect/src/request_handler/ton.rs) |
| Tron | ✅ | `tron` | [Tron (3)](../core/crates/gem_wallet_connect/src/request_handler/tron.rs) |
| Dogecoin | ➖ | ➖ | ➖ |
| Zcash | ➖ | ➖ | ➖ |
| Optimism | ✅ | `eip155` | [EVM (8)](../core/crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Aptos | ➖ | ➖ | ➖ |
| Base | ✅ | `eip155` | [EVM (8)](../core/crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Avalanche C-Chain | ✅ | `eip155` | [EVM (8)](../core/crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Sui | ✅ | `sui` | [Sui (4)](../core/crates/gem_wallet_connect/src/request_handler/sui.rs) |
| XRP Ledger | ➖ | ➖ | ➖ |
| opBNB | ✅ | `eip155` | [EVM (8)](../core/crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Fantom | ✅ | `eip155` | [EVM (8)](../core/crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Gnosis | ✅ | `eip155` | [EVM (8)](../core/crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Celestia | ➖ | ➖ | ➖ |
| Injective | ➖ | ➖ | ➖ |
| Sei | ➖ | ➖ | ➖ |
| Sei EVM | ✅ | `eip155` | [EVM (8)](../core/crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Manta | ✅ | `eip155` | [EVM (8)](../core/crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Blast | ✅ | `eip155` | [EVM (8)](../core/crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Noble | ➖ | ➖ | ➖ |
| ZKsync | ✅ | `eip155` | [EVM (8)](../core/crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Linea | ✅ | `eip155` | [EVM (8)](../core/crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Mantle | ✅ | `eip155` | [EVM (8)](../core/crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Celo | ✅ | `eip155` | [EVM (8)](../core/crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| NEAR | ➖ | ➖ | ➖ |
| World Chain | ✅ | `eip155` | [EVM (8)](../core/crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Stellar | ➖ | ➖ | ➖ |
| Sonic | ✅ | `eip155` | [EVM (8)](../core/crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Algorand | ➖ | ➖ | ➖ |
| Polkadot | ➖ | ➖ | ➖ |
| Plasma | ✅ | `eip155` | [EVM (8)](../core/crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Cardano | ➖ | ➖ | ➖ |
| Abstract | ✅ | `eip155` | [EVM (8)](../core/crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Berachain | ✅ | `eip155` | [EVM (8)](../core/crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Ink | ✅ | `eip155` | [EVM (8)](../core/crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Unichain | ✅ | `eip155` | [EVM (8)](../core/crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Hyperliquid | ✅ | `eip155` | [EVM (8)](../core/crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| HyperCore | ➖ | ➖ | ➖ |
| Monad | ✅ | `eip155` | [EVM (8)](../core/crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| X Layer | ✅ | `eip155` | [EVM (8)](../core/crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Robinhood Chain | ✅ | `eip155` | [EVM (8)](../core/crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Stable | ✅ | `eip155` | [EVM (8)](../core/crates/gem_wallet_connect/src/request_handler/ethereum.rs) |
| Tempo | ✅ | `eip155` | [EVM (8)](../core/crates/gem_wallet_connect/src/request_handler/ethereum.rs) |

<sub>Reviewed 2026-07-21. Sources: [advertised chains](../core/gemstone/src/config/wallet_connect.rs), [method identifiers](../core/crates/primitives/src/wallet_connector.rs), [Core dispatcher](../core/crates/gem_wallet_connect/src/request_handler/mod.rs), [Android namespaces](../android/gemcore/src/main/kotlin/com/gemwallet/android/application/wallet_connect/Namespace.kt), and [iOS method advertisement](../ios/Features/WalletConnector/Sources/WalletConnector/Services/WalletConnectorSigner.swift).</sub>

## Transaction-indexing providers

The EVM route configured by [`settings_chain`](../core/crates/settings_chain/src/lib.rs) uses ordered provider lists. Blockscout PRO is first for its 12 supported Gem chains, followed by the existing Ankr or Alchemy route when Blockscout returns an error. Alchemy's [`alchemy_getAssetTransfers`](../core/crates/gem_evm/src/rpc/alchemy/client.rs) is only called on explicitly supported chains. Other EVM chains return an empty address history. Dedicated indexers for Algorand, EVM, NEAR, Polkadot, Solana, Sui, and Tron are composed only by `settings_chain`; [gemstone](../core/gemstone/src/gateway/chain_factory.rs) uses RPC-only providers, and swapper uses raw RPC clients.

| Gem chain | Providers |
| --- | --- |
| Bitcoin | [Bitcoin chain client](../core/crates/gem_bitcoin/src/rpc/client.rs) |
| Bitcoin Cash | [Bitcoin chain client](../core/crates/gem_bitcoin/src/rpc/client.rs) |
| Litecoin | [Bitcoin chain client](../core/crates/gem_bitcoin/src/rpc/client.rs) |
| Ethereum | [Blockscout](../core/crates/gem_evm/src/rpc/blockscout/client.rs), [Ankr](../core/crates/gem_evm/src/rpc/ankr/client.rs) |
| BNB Smart Chain | [Ankr](../core/crates/gem_evm/src/rpc/ankr/client.rs) |
| Solana | [Alchemy](../core/crates/gem_solana/src/rpc/indexer/mod.rs) |
| Polygon | [Blockscout](../core/crates/gem_evm/src/rpc/blockscout/client.rs), [Ankr](../core/crates/gem_evm/src/rpc/ankr/client.rs) |
| THORChain | [Cosmos chain client](../core/crates/gem_cosmos/src/rpc/client.rs) |
| MayaChain | [Cosmos chain client](../core/crates/gem_cosmos/src/rpc/client.rs) |
| Cosmos | [Cosmos chain client](../core/crates/gem_cosmos/src/rpc/client.rs) |
| Osmosis | [Cosmos chain client](../core/crates/gem_cosmos/src/rpc/client.rs) |
| Arbitrum | [Blockscout](../core/crates/gem_evm/src/rpc/blockscout/client.rs), [Ankr](../core/crates/gem_evm/src/rpc/ankr/client.rs) |
| TON | [TON chain client](../core/crates/gem_ton/src/rpc/client.rs) |
| Tron | [TronGrid](../core/crates/gem_tron/src/rpc/trongrid/client.rs) |
| Dogecoin | [Bitcoin chain client](../core/crates/gem_bitcoin/src/rpc/client.rs) |
| Zcash | [Bitcoin chain client](../core/crates/gem_bitcoin/src/rpc/client.rs) |
| Optimism | [Blockscout](../core/crates/gem_evm/src/rpc/blockscout/client.rs), [Ankr](../core/crates/gem_evm/src/rpc/ankr/client.rs) |
| Aptos | [Aptos chain client](../core/crates/gem_aptos/src/rpc/client.rs) |
| Base | [Blockscout](../core/crates/gem_evm/src/rpc/blockscout/client.rs), [Ankr](../core/crates/gem_evm/src/rpc/ankr/client.rs) |
| Avalanche C-Chain | [Ankr](../core/crates/gem_evm/src/rpc/ankr/client.rs) |
| Sui | [Sui GraphQL](../core/crates/gem_sui/src/rpc/indexer/mod.rs) |
| XRP Ledger | [XRP chain client](../core/crates/gem_xrp/src/rpc/client.rs) |
| opBNB | Unsupported |
| Fantom | [Ankr](../core/crates/gem_evm/src/rpc/ankr/client.rs) |
| Gnosis | [Blockscout](../core/crates/gem_evm/src/rpc/blockscout/client.rs), [Ankr](../core/crates/gem_evm/src/rpc/ankr/client.rs) |
| Celestia | [Cosmos chain client](../core/crates/gem_cosmos/src/rpc/client.rs) |
| Injective | [Cosmos chain client](../core/crates/gem_cosmos/src/rpc/client.rs) |
| Sei | [Cosmos chain client](../core/crates/gem_cosmos/src/rpc/client.rs) |
| Sei EVM | Unsupported |
| Manta | Unsupported |
| Blast | [Alchemy](../core/crates/gem_evm/src/rpc/alchemy/client.rs) |
| Noble | [Cosmos chain client](../core/crates/gem_cosmos/src/rpc/client.rs) |
| ZKsync | [Blockscout](../core/crates/gem_evm/src/rpc/blockscout/client.rs), [Alchemy](../core/crates/gem_evm/src/rpc/alchemy/client.rs) |
| Linea | [Ankr](../core/crates/gem_evm/src/rpc/ankr/client.rs) |
| Mantle | Unsupported |
| Celo | [Blockscout](../core/crates/gem_evm/src/rpc/blockscout/client.rs), [Alchemy](../core/crates/gem_evm/src/rpc/alchemy/client.rs) |
| NEAR | [FastNear](../core/crates/gem_near/src/rpc/indexer/mod.rs) |
| World Chain | [Blockscout](../core/crates/gem_evm/src/rpc/blockscout/client.rs), [Alchemy](../core/crates/gem_evm/src/rpc/alchemy/client.rs) |
| Stellar | [Stellar chain client](../core/crates/gem_stellar/src/rpc/client.rs) |
| Sonic | Unsupported |
| Algorand | [Algorand Indexer](../core/crates/gem_algorand/src/rpc/indexer/mod.rs) |
| Polkadot | [Subscan](../core/crates/gem_polkadot/src/rpc/indexer/mod.rs) |
| Plasma | Unsupported |
| Cardano | [Cardano chain client](../core/crates/gem_cardano/src/rpc/client.rs) |
| Abstract | [Alchemy](../core/crates/gem_evm/src/rpc/alchemy/client.rs) |
| Berachain | [Alchemy](../core/crates/gem_evm/src/rpc/alchemy/client.rs) |
| Ink | [Blockscout](../core/crates/gem_evm/src/rpc/blockscout/client.rs), [Alchemy](../core/crates/gem_evm/src/rpc/alchemy/client.rs) |
| Unichain | [Blockscout](../core/crates/gem_evm/src/rpc/blockscout/client.rs), [Alchemy](../core/crates/gem_evm/src/rpc/alchemy/client.rs) |
| Hyperliquid | [Alchemy](../core/crates/gem_evm/src/rpc/alchemy/client.rs) |
| HyperCore | [HyperCore chain client](../core/crates/gem_hypercore/src/provider/transactions.rs) |
| Monad | [Alchemy](../core/crates/gem_evm/src/rpc/alchemy/client.rs) |
| X Layer | [Ankr](../core/crates/gem_evm/src/rpc/ankr/client.rs) |
| Robinhood Chain | [Blockscout](../core/crates/gem_evm/src/rpc/blockscout/client.rs), [Alchemy](../core/crates/gem_evm/src/rpc/alchemy/client.rs) |
| Stable | Unsupported |
| Tempo | Unsupported |

[Ankr's Advanced API](https://www.ankr.com/docs/advanced-api/overview/) also advertises Flare, Scroll, Story, Syscoin, Taiko, Telos, and Xai, which are not Gem chains. Alchemy's generic [Chain API list](https://www.alchemy.com/docs/reference/node-supported-chains) is retained as endpoint evidence only; method-level support must be checked against the [Transfers API](https://www.alchemy.com/docs/reference/transfers-api-quickstart) or an authenticated request.

<sub>Reviewed 2026-07-21. External method references: [Blockscout PRO chains](https://docs.blockscout.com/devs/pro-api) · [Blockscout transactions](https://docs.blockscout.com/api-reference/get-address-transactions) · [Blockscout token transfers](https://docs.blockscout.com/api-reference/get-address-token-transfers) · [Blockscout token balances](https://docs.blockscout.com/api-reference/get-all-tokens-balances-for-the-address) · [Ankr Advanced API](https://www.ankr.com/docs/advanced-api/overview/) · [Alchemy Transfers API](https://www.alchemy.com/docs/reference/transfers-api-quickstart) · [Alchemy Solana history method](https://www.alchemy.com/docs/chains/solana/solana-api-endpoints/get-transactions-for-address) · [Sui GraphQL](https://docs.sui.io/concepts/data-access/graphql-rpc) · [TronGrid account transactions](https://developers.tron.network/reference/get-transaction-info-by-account-address) · [Algorand Indexer](https://dev.algorand.co/reference/rest-api/indexer/).</sub>

## Swap providers

These tables compare Core route eligibility with current provider-advertised support. A Core chain means that the provider's `supported_assets()` exposes at least one asset on that chain; it does not mean that every token or chain pair is available. `Coverage difference` only mentions chains in Gem's 55-chain [`Chain`](../core/crates/primitives/src/chain.rs) enum.

### On-chain providers

| Provider | Core coverage | Core chains | Coverage difference | References |
| --- | :---: | --- | --- | --- |
| [Uniswap v3](../core/crates/swapper/src/uniswap/v3/provider.rs) | 16/55 | Ethereum, Optimism, Arbitrum, Polygon, Avalanche C-Chain, Base, BNB Smart Chain, ZKsync, Celo, Blast, World Chain, Unichain, Monad, X Layer, Stable, Robinhood Chain | Core-only relative to the current official list: Blast, Stable | <sub>[code](../core/crates/gem_evm/src/uniswap/deployment/v3.rs) · [current deployments](https://developers.uniswap.org/docs/protocols/v3/deployments) · [Blast sunset](https://developers.uniswap.org/docs/changelog/active-notifications/sunset-of-blast-support) · [Stable deployment](https://swap.stable.xyz/deployments)</sub> |
| [Uniswap v4](../core/crates/swapper/src/uniswap/v4/provider.rs) | 17/55 | Ethereum, Optimism, Arbitrum, Polygon, Avalanche C-Chain, Base, BNB Smart Chain, Blast, Linea, World Chain, Unichain, Celo, Monad, Ink, X Layer, Robinhood Chain, Tempo | ➖ | <sub>[code](../core/crates/gem_evm/src/uniswap/deployment/v4.rs) · [deployments](https://github.com/Uniswap/contracts/blob/main/deployments/index.md)</sub> |
| [PancakeSwap v3](../core/crates/swapper/src/uniswap/v3/provider.rs) | 9/55 | Ethereum, BNB Smart Chain, opBNB, Arbitrum, Linea, Base, ZKsync, Monad, Robinhood Chain | ➖ | <sub>[code](../core/crates/gem_evm/src/uniswap/deployment/v3.rs) · [v3 router deployments](https://developer.pancakeswap.finance/contracts/universal-router/addresses)</sub> |
| [OKX DEX](../core/crates/swapper/src/okx/provider.rs) | 22/55 | Solana, Tron, Ethereum, BNB Smart Chain, Polygon, Arbitrum, Optimism, Base, Avalanche C-Chain, Fantom, Manta, Blast, ZKsync, Linea, Mantle, Plasma, Hyperliquid, Sonic, Unichain, Monad, X Layer, Robinhood Chain | Missing: Sui, TON | <sub>[code](../core/crates/swapper/src/okx/provider.rs) · [chains API](https://web3.okx.com/onchainos/dev-docs-v5/dex-api/dex-get-aggregator-supported-chains)</sub> |
| [Oku](../core/crates/swapper/src/uniswap/v3/provider.rs) | 5/55 | Sonic, Mantle, Gnosis, Plasma, Sei EVM | Not assigned to Oku in Core: Ethereum, Optimism, Arbitrum, Polygon, ZKsync, Base, BNB Smart Chain, Avalanche C-Chain, Blast, Linea, Celo, Manta, Unichain, World Chain, Monad | <sub>[code](../core/crates/gem_evm/src/uniswap/deployment/v3.rs) · [deployments](https://docs.oku.trade/home/extra-information/deployed-contracts)</sub> |
| [Wagmi](../core/crates/swapper/src/uniswap/v3/provider.rs) | 1/55 | Sonic | Missing: Ethereum, BNB Smart Chain, Avalanche C-Chain, Polygon, Fantom, Arbitrum, Optimism, Base | <sub>[code](../core/crates/gem_evm/src/uniswap/deployment/v3.rs) · [contracts](https://docs.wagmi.com/wagmi/contracts)</sub> |
| [Aerodrome](../core/crates/swapper/src/uniswap/v3/provider.rs) | 1/55 | Base | ➖ | <sub>[code](../core/crates/gem_evm/src/uniswap/deployment/v3.rs) · [contracts](https://aerodrome.finance/security)</sub> |
| [Jupiter](../core/crates/swapper/src/jupiter/provider.rs) | 1/55 | Solana | ➖ | <sub>[code](../core/crates/swapper/src/jupiter/provider.rs) · [quote API](https://dev.jup.ag/docs/swap/v1/get-quote)</sub> |
| [Panora](../core/crates/swapper/src/panora/provider.rs) | 1/55 | Aptos | ➖ | <sub>[code](../core/crates/swapper/src/panora/provider.rs) · [swap API](https://docs.panora.exchange/developer/swap/api)</sub> |
| [STON.fi v2](../core/crates/swapper/src/stonfi/provider.rs) | 1/55 | TON | ➖ | <sub>[code](../core/crates/swapper/src/stonfi/provider.rs) · [DEX API](https://docs.ston.fi/developer-section/dex/api/reference)</sub> |
| [Cetus CLMM](../core/crates/swapper/src/cetus_clmm/provider.rs) | 1/55 | Sui | Missing: Aptos | <sub>[code](../core/crates/swapper/src/cetus_clmm/provider.rs) · [Sui developer docs](https://cetus-1.gitbook.io/cetus-developer-docs) · [Cetus chain support](https://cetus-1.gitbook.io/cetus-docs/guides/faq)</sub> |

<sub>Additional OKX gap evidence: [Sui guide](https://web3.okx.com/onchainos/dev-docs-v5/dex-api/dex-use-swap-sui-quick-start) and [TON guide](https://web3.okx.com/build/docs/waas/dex-use-swap-ton-quick-start).</sub>

### Cross-chain and bridge providers

| Provider | Mode | Core coverage | Core chains | Coverage difference | References |
| --- | --- | :---: | --- | --- | --- |
| [NEAR Intents](../core/crates/swapper/src/near_intents/provider.rs) | Omnichain | 27/55 | NEAR, Ethereum, Bitcoin, Solana, Sui, Arbitrum, Base, Optimism, Avalanche C-Chain, BNB Smart Chain, Polygon, TON, Tron, Dogecoin, XRP Ledger, Cardano, Berachain, Aptos, Zcash, Gnosis, Stellar, Litecoin, Bitcoin Cash, Monad, X Layer, Plasma, Abstract | Missing: HyperCore | <sub>[code](../core/crates/swapper/src/near_intents/assets.rs) · [live token API](https://1click.chaindefuser.com/v0/tokens) · [chain support](https://docs.near-intents.org/resources/chain-support)</sub> |
| [Relay](../core/crates/swapper/src/relay/provider.rs) | Omnichain | 29/55 | Ethereum, BNB Smart Chain, Base, Arbitrum, Optimism, Polygon, Avalanche C-Chain, Linea, ZKsync, Hyperliquid, Sei EVM, Berachain, Manta, Sonic, Abstract, Celo, Stable, Robinhood Chain, Gnosis, Mantle, Blast, World Chain, Ink, Unichain, Monad, Plasma, Tempo, Tron, Solana | Missing: Bitcoin, TON, HyperCore | <sub>[code](../core/crates/swapper/src/relay/asset.rs) · [live chains API](https://api.relay.link/chains)</sub> |
| [Across](../core/crates/swapper/src/across/provider.rs) | Bridge | 17/55 | Ethereum, Optimism, Polygon, Arbitrum, Avalanche C-Chain, Base, Hyperliquid, Linea, ZKsync, World Chain, Ink, Unichain, Monad, BNB Smart Chain, Plasma, Robinhood Chain, Tron | Missing: Solana, HyperCore; 🪦 Blast | <sub>[code](../core/crates/gem_evm/src/across/deployment.rs) · [chains and contracts](https://docs.across.to/chains-and-contracts) · [live chains API](https://app.across.to/api/swap/chains)</sub> |
| [THORChain](../core/crates/swapper/src/thorchain/provider.rs) | Omnichain | 14/55 | Dogecoin, THORChain, Ethereum, Cosmos, Bitcoin, Bitcoin Cash, Litecoin, BNB Smart Chain, Avalanche C-Chain, Base, XRP Ledger, Tron, Solana, Zcash | Core-only: Zcash | <sub>[code](../core/crates/swapper/src/thorchain/chain.rs) · [supported chains](https://dev.thorchain.org/concepts/querying-thorchain.html) · [live inbound-address API](https://gateway.liquify.com/chain/thorchain_api/thorchain/inbound_addresses)</sub> |
| [Mayan](../core/crates/swapper/src/mayan/provider.rs) | Cross-chain | 14/55 | Ethereum, Solana, Sui, BNB Smart Chain, Base, Polygon, Avalanche C-Chain, Arbitrum, Optimism, Linea, Unichain, Monad, Hyperliquid, HyperCore | ➖ | <sub>[code](../core/crates/swapper/src/mayan/asset.rs) · [quote API](https://docs.mayan.finance/integration/quote-api) · [live configuration](https://sia.mayan.finance/v10/init)</sub> |
| [Squid](../core/crates/swapper/src/squid/provider.rs) | Cross-chain | 6/55 | Cosmos, Osmosis, Celestia, Injective, Sei, Noble | Missing: Ethereum, BNB Smart Chain, Arbitrum, Optimism, Polygon, Avalanche C-Chain, Base, Fantom, Linea, Mantle, Celo, Blast, Berachain, Gnosis, Sonic, Hyperliquid, Bitcoin, Solana, Sui, XRP Ledger, Stellar | <sub>[code](../core/crates/swapper/src/squid/provider.rs) · [supported chains](https://docs.squidrouter.com/api-and-sdk-integration/key-concepts/get-supported-tokens-and-chains)</sub> |
| [MayaChain](../core/crates/swapper/src/thorchain/provider.rs) | Cross-chain | 6/55 | THORChain, Bitcoin, Ethereum, Arbitrum, Zcash, Cardano | ➖ | <sub>[code](../core/crates/swapper/src/thorchain/chain.rs) · [live chains API](https://mayanode.mayachain.info/mayachain/inbound_addresses)</sub> |
| [Chainflip](../core/crates/swapper/src/chainflip/provider.rs) | Cross-chain | 5/55 | Bitcoin (destination only), Ethereum, Solana, Tron, Arbitrum | Missing: Polkadot | <sub>[asset mapping](../core/crates/swapper/src/chainflip/client/model.rs) · [source-chain guard and cached minimum amounts](../core/crates/swapper/src/chainflip/provider.rs) · [Broker native quotes](https://docs.chainflip-broker.io/features/ask-quote/) · [live assets API](https://docs.chainflip-broker.io/features/view-assets/) · [supported chains](https://docs.chainflip.io/protocol/supported-chains-assets/chains-assets)</sub> |
| [Swaps.xyz](../core/crates/swapper/src/swaps_xyz/provider.rs) | Cross-chain | 10/55 | Algorand, Stellar, Cardano, TON, Cosmos, Osmosis, Aptos, Sui, XRP, Tron | Native assets only | <sub>[code](../core/crates/swapper/src/swaps_xyz/provider.rs) · [API introduction](https://docs.swaps.xyz/)</sub> |
| [Hyperliquid](../core/crates/swapper/src/hyperliquid/provider/hyperliquid.rs) | Omnichain | 2/55 | HyperCore, Hyperliquid | ➖ | <sub>[code](../core/crates/swapper/src/hyperliquid/provider/hyperliquid.rs) · [Core ↔ EVM transfers](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/hyperevm/hypercore-less-than-greater-than-hyperevm-transfers)</sub> |

### Provider behavior

| Provider | Type | Amount | Slippage | Status tracking | Vault discovery |
| --- | --- | --- | --- | :---: | :---: |
| [Uniswap v3](../core/crates/swapper/src/uniswap/v3/provider.rs) | On-chain | Fixed | Exact | ➖ | ➖ |
| [Uniswap v4](../core/crates/swapper/src/uniswap/v4/provider.rs) | On-chain | Fixed | Exact | ➖ | ➖ |
| [PancakeSwap v3](../core/crates/swapper/src/uniswap/v3/provider.rs) | On-chain | Fixed | Exact | ➖ | ➖ |
| [OKX DEX](../core/crates/swapper/src/okx/provider.rs) | On-chain | Flexible | Auto | ➖ | ➖ |
| [Oku](../core/crates/swapper/src/uniswap/v3/provider.rs) | On-chain | Fixed | Exact | ➖ | ➖ |
| [Wagmi](../core/crates/swapper/src/uniswap/v3/provider.rs) | On-chain | Fixed | Exact | ➖ | ➖ |
| [Aerodrome](../core/crates/swapper/src/uniswap/v3/provider.rs) | On-chain | Fixed | Exact | ➖ | ➖ |
| [Jupiter](../core/crates/swapper/src/jupiter/provider.rs) | On-chain | Fixed | Exact | ➖ | ➖ |
| [Panora](../core/crates/swapper/src/panora/provider.rs) | On-chain | Fixed | Exact | ➖ | ➖ |
| [STON.fi v2](../core/crates/swapper/src/stonfi/provider.rs) | On-chain | Fixed | Exact | ➖ | ➖ |
| [Cetus CLMM](../core/crates/swapper/src/cetus_clmm/provider.rs) | On-chain | Fixed | Exact | ➖ | ➖ |
| [NEAR Intents](../core/crates/swapper/src/near_intents/provider.rs) | Omnichain | Flexible | Exact | ✅ | ✅ |
| [Relay](../core/crates/swapper/src/relay/provider.rs) | Omnichain | Fixed | Auto | ✅ | ✅ |
| [Across](../core/crates/swapper/src/across/provider.rs) | Bridge | Fixed | Exact | ✅ | ✅ |
| [THORChain](../core/crates/swapper/src/thorchain/provider.rs) | Omnichain | Fixed from EVM; flexible otherwise | Exact | ✅ | ✅ |
| [Mayan](../core/crates/swapper/src/mayan/provider.rs) | Cross-chain | Fixed | Auto | ✅ | ✅ |
| [Squid](../core/crates/swapper/src/squid/provider.rs) | Cross-chain | Fixed | Auto | ✅ | ✅ |
| [MayaChain](../core/crates/swapper/src/thorchain/provider.rs) | Cross-chain | Fixed from EVM; flexible otherwise | Exact | ✅ | ✅ |
| [Chainflip](../core/crates/swapper/src/chainflip/provider.rs) | Cross-chain | Fixed | Exact | ✅ | ✅ |
| [Swaps.xyz](../core/crates/swapper/src/swaps_xyz/provider.rs) | Cross-chain | Fixed | Exact | ✅ | ✅ (quoted address) |
| [Hyperliquid](../core/crates/swapper/src/hyperliquid/provider/hyperliquid.rs) | Omnichain | Flexible | Exact | ➖ (assumed complete) | ➖ |

<sub>Sources: [provider modes and slippage](../core/crates/swapper/src/models.rs), [amount-mode implementations](../core/crates/swapper/src), and [active provider registry](../core/crates/swapper/src/swapper.rs).</sub>

Provider-specific status tracking and vault discovery are `➖` for on-chain routes; transaction tracking belongs to the chain client.

`Omnichain` providers may be eligible for selected same-chain routes as well as cross-chain routes. `Bridge` and `Cross-chain` providers require different source and destination chains.

<sub>Reviewed 2026-07-22. Active-provider source of truth: [`GemSwapper::new`](../core/crates/swapper/src/swapper.rs). `CetusAggregator` and `Orca` remain inactive [`SwapProvider`](../core/crates/primitives/src/swap_provider.rs) variants.</sub>

## Fiat providers

The active registry constructs all six providers below. `Order lookup` means Core can poll a provider order directly; webhook processing is supported independently for every active provider.

| Provider | Buy quotes | Sell quotes | Asset catalog | Countries | Checkout | Webhooks | Order lookup |
| --- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| [MoonPay](../core/crates/fiat/src/providers/moonpay/provider.rs) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| [Mercuryo](../core/crates/fiat/src/providers/mercuryo/provider.rs) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| [Transak](../core/crates/fiat/src/providers/transak/provider.rs) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| [Banxa](../core/crates/fiat/src/providers/banxa/provider.rs) | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ |
| [Paybis](../core/crates/fiat/src/providers/paybis/provider.rs) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| [Cash App / Flashnet](../core/crates/fiat/src/providers/flashnet/provider.rs) | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ |

Card, Apple Pay, and Google Pay are the default payment methods. Flashnet overrides that list with Cash App. Country data is provider-backed except Paybis, which uses Core's country-status mapping, and Flashnet, which is US-only.

<sub>Reviewed 2026-07-21. Sources: [active provider factory](../core/crates/fiat/src/lib.rs), [provider contract](../core/crates/fiat/src/provider.rs), and [Paybis payment methods](../core/crates/fiat/src/providers/paybis/mapper.rs).</sub>

## NFT providers

The NFT registry selects providers by chain and falls through in registration order when a provider request fails. Wallet assets, collection metadata, and individual asset metadata are part of the shared provider contract.

| Provider | Active | Chains | Wallet assets | Collections | Asset details | Notes |
| --- | :---: | --- | :---: | :---: | :---: | --- |
| [OpenSea](../core/crates/nft/src/providers/opensea/provider.rs) | ✅ | Ethereum, Polygon | ✅ | ✅ | ✅ | First registered provider |
| [Magic Eden Solana](../core/crates/nft/src/providers/magiceden/solana/provider.rs) | ✅ | Solana | ✅ | ✅ | ✅ | Dedicated Solana API |
| [Alchemy NFT](../core/crates/nft/src/providers/alchemy/provider.rs) | ✅ | BNB Smart Chain | ✅ | ✅ | ✅ | Filters assets classified as spam |
| [TON](../core/crates/nft/src/providers/ton/provider.rs) | ✅ | TON | ✅ | ✅ | ✅ | Uses indexed on-chain data with off-chain metadata fallback |

<sub>Reviewed 2026-08-22. Sources: [active provider factory](../core/crates/nft/src/factory.rs), [provider contract and fallback behavior](../core/crates/nft/src/provider.rs), [supported NFT chains](../core/crates/primitives/src/chain_nft.rs), and [NFT settings](../core/crates/settings/src/lib.rs).</sub>

## Actionable TODOs

This backlog only includes gaps that apply to an existing Gem chain, an active provider, or a current cross-platform inconsistency. Non-applicable and speculative capabilities—such as Bitcoin simulation or Bitcoin NFTs—and provider chains that Gem does not support are intentionally excluded.

| Area | Work | Applicable scope | References | Status |
| --- | --- | --- | --- | :---: |
| WalletConnect | Align EVM method policy: remove `eth_sendRawTransaction` from platform advertisements or implement it in Core, and explicitly decide whether to support `eth_sign` | EVM chains | <sub>[Core handler](../core/crates/gem_wallet_connect/src/request_handler/ethereum.rs) · [EVM method reference](https://docs.walletconnect.network/wallet-sdk/chain-support/evm) · [Android](../android/gemcore/src/main/kotlin/com/gemwallet/android/application/wallet_connect/Namespace.kt) · [iOS](../ios/Features/WalletConnector/Sources/WalletConnector/Services/WalletConnectorSigner.swift)</sub> | 🏗️ |
| WalletConnect | Add the published Bitcoin namespace and request handler | Bitcoin | <sub>[Core dispatcher](../core/crates/gem_wallet_connect/src/request_handler/mod.rs) · [WalletConnect Bitcoin spec](https://docs.walletconnect.network/wallet-sdk/chain-support/bitcoin)</sub> | 🏗️ |
| Oku | Add compatible provider deployment mappings not already assigned to the Oku adapter | Ethereum, Optimism, Arbitrum, Polygon, ZKsync, Base, BNB Smart Chain, Avalanche C-Chain, Blast, Linea, Celo, Manta, Unichain, World Chain, Monad | <sub>[Core deployment](../core/crates/gem_evm/src/uniswap/deployment/v3.rs) · [provider deployments](https://docs.oku.trade/home/extra-information/deployed-contracts)</sub> | 🏗️ |
| Wagmi | Add the remaining documented provider deployments | Ethereum, BNB Smart Chain, Avalanche C-Chain, Polygon, Fantom, Arbitrum, Optimism, Base | <sub>[Core deployment](../core/crates/gem_evm/src/uniswap/deployment/v3.rs) · [provider contracts](https://docs.wagmi.com/wagmi/contracts)</sub> | 🏗️ |
| Cetus CLMM | Add an Aptos-specific quote and transaction implementation | Aptos | <sub>[Core Sui provider](../core/crates/swapper/src/cetus_clmm/provider.rs) · [Cetus chain support](https://cetus-1.gitbook.io/cetus-docs/guides/faq)</sub> | 🏗️ |
| OKX DEX | Add the Sui quote and transaction path | Sui | <sub>[Core provider](../core/crates/swapper/src/okx/provider.rs) · [Sui guide](https://web3.okx.com/onchainos/dev-docs-v5/dex-api/dex-use-swap-sui-quick-start)</sub> | 🏗️ |
| OKX DEX | Add the TON quote and transaction path | TON | <sub>[Core provider](../core/crates/swapper/src/okx/provider.rs) · [TON guide](https://web3.okx.com/build/docs/waas/dex-use-swap-ton-quick-start)</sub> | 🏗️ |
| Relay | Add non-EVM quote and transaction implementations before exposing the remaining provider chains | Bitcoin, Solana, TON, HyperCore | <sub>[Core transaction model](../core/crates/swapper/src/relay/model.rs) · [live chains API](https://api.relay.link/chains)</sub> | 🏗️ |
| Across | Add non-EVM quote and transaction implementations | Solana, HyperCore | <sub>[Core provider](../core/crates/swapper/src/across/provider.rs) · [live chains API](https://app.across.to/api/swap/chains)</sub> | 🏗️ |
| Squid | Add an EVM quote and transaction implementation | Ethereum, BNB Smart Chain, Arbitrum, Optimism, Polygon, Avalanche C-Chain, Base, Fantom, Linea, Mantle, Celo, Blast, Berachain, Gnosis, Sonic, Hyperliquid | <sub>[Core Cosmos transaction path](../core/crates/swapper/src/squid/provider.rs) · [supported chains](https://docs.squidrouter.com/api-and-sdk-integration/key-concepts/get-supported-tokens-and-chains)</sub> | 🏗️ |
| Squid | Add the corresponding non-EVM quote and transaction implementations | Bitcoin, Solana, Sui, XRP Ledger, Stellar | <sub>[Core provider](../core/crates/swapper/src/squid/provider.rs) · [supported chains](https://docs.squidrouter.com/api-and-sdk-integration/key-concepts/get-supported-tokens-and-chains)</sub> | 🏗️ |
| Chainflip | Add the Bitcoin source transaction path; it is currently destination-only | Bitcoin | <sub>[source-chain guard](../core/crates/swapper/src/chainflip/provider.rs) · [Core asset mapping](../core/crates/swapper/src/chainflip/client/model.rs) · [provider chains](https://docs.chainflip.io/protocol/supported-chains-assets/chains-assets)</sub> | 🏗️ |
| Chainflip | Add the provider-supported relay-chain asset and transaction path | Polkadot | <sub>[Core asset mapping](../core/crates/swapper/src/chainflip/client/model.rs) · [provider chains](https://docs.chainflip.io/protocol/supported-chains-assets/chains-assets)</sub> | 🏗️ |
| THORChain | Remove or disable the Core-only route unless it becomes active in THORChain | Zcash | <sub>[Core mapping](../core/crates/swapper/src/thorchain/chain.rs) · [supported chains](https://dev.thorchain.org/concepts/querying-thorchain.html) · [live chains API](https://gateway.liquify.com/chain/thorchain_api/thorchain/inbound_addresses)</sub> | 🏗️ |
| NEAR Intents | Add the provider-advertised asset mapping | HyperCore | <sub>[Core mapping](../core/crates/swapper/src/near_intents/assets.rs) · [live token API](https://1click.chaindefuser.com/v0/tokens)</sub> | 🏗️ |
