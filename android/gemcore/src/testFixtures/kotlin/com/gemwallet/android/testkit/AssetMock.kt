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

fun mockAssetEthereum() = mockAsset(
    chain = Chain.Ethereum,
    name = "Ethereum",
    symbol = "ETH",
    decimals = 18,
)
