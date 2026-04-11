package com.gemwallet.android.testkit

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain

fun mockAsset(
    chain: Chain = Chain.Bitcoin,
    tokenId: String? = null,
    name: String = "Bitcoin",
    symbol: String = "BTC",
    decimals: Int = 8,
    type: AssetType = AssetType.NATIVE,
) = Asset(
    id = AssetId(chain, tokenId),
    name = name,
    symbol = symbol,
    decimals = decimals,
    type = type,
)

fun mockAssetSolana() = mockAsset(
    chain = Chain.Solana,
    name = "Solana",
    symbol = "SOL",
    decimals = 9,
)

fun mockAssetSolanaUSDC() = mockAsset(
    chain = Chain.Solana,
    tokenId = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    name = "USD Coin",
    symbol = "USDC",
    decimals = 6,
    type = AssetType.SPL,
)

fun mockAssetEthereum() = mockAsset(
    chain = Chain.Ethereum,
    name = "Ethereum",
    symbol = "ETH",
    decimals = 18,
)

fun mockAssetMonad() = mockAsset(
    chain = Chain.Monad,
    name = "Monad",
    symbol = "MON",
    decimals = 18,
)

fun mockAssetCosmos() = mockAsset(
    chain = Chain.Cosmos,
    name = "Cosmos",
    symbol = "ATOM",
    decimals = 6,
)

fun mockAssetTron() = mockAsset(
    chain = Chain.Tron,
    name = "Tron",
    symbol = "TRX",
    decimals = 6,
)

fun mockAssetHyperCoreHype() = mockAsset(
    chain = Chain.HyperCore,
    tokenId = "HYPE::0x0d01dc56dcaaca66ad901c959b4011ec::150",
    name = "Hyperliquid",
    symbol = "HYPE",
    decimals = 8,
    type = AssetType.TOKEN,
)

fun mockAssetHyperCoreUSDC() = mockAsset(
    chain = Chain.HyperCore,
    tokenId = "USDC::0x6d1e7cde53ba9467b783cb7c530ce054::0",
    name = "USDC",
    symbol = "USDC",
    decimals = 8,
    type = AssetType.TOKEN,
)

fun mockAssetHyperCoreUBTC() = mockAsset(
    chain = Chain.HyperCore,
    tokenId = "UBTC::0x8f254b963e8468305d409b33aa137c67::197",
    name = "Bitcoin",
    symbol = "UBTC",
    decimals = 10,
    type = AssetType.TOKEN,
)
