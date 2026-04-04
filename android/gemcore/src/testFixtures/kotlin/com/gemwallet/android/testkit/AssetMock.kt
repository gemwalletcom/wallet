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
