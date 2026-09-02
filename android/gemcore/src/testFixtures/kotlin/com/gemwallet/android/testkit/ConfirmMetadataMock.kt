package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemAssetBalance
import uniffi.gemstone.GemAssetPrice
import uniffi.gemstone.GemConfirmMetadata

fun mockGemAssetBalance(
    asset: Asset = mockAssetEthereum(),
    available: String = "0",
) = GemAssetBalance(
    assetId = asset.id.toIdentifier(),
    available = available,
    frozen = "0",
    locked = "0",
    staked = "0",
    pending = "0",
    pendingUnconfirmed = "0",
    rewards = "0",
    reserved = "0",
    withdrawable = "0",
    earn = "0",
    metadata = null,
)

fun mockGemConfirmMetadata(
    asset: Asset = mockAssetEthereum(),
    feeAsset: Asset = asset,
    prices: List<GemAssetPrice> = emptyList(),
) = GemConfirmMetadata(
    assetBalance = mockGemAssetBalance(asset),
    feeAssetBalance = mockGemAssetBalance(feeAsset),
    prices = prices,
)
