package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemAssetBalance
import uniffi.gemstone.GemAssetPrice
import uniffi.gemstone.GemConfirmMetadata
import java.math.BigInteger

fun mockGemAssetBalance(
    asset: Asset = mockAssetEthereum(),
    available: BigInteger = BigInteger.ZERO,
) = GemAssetBalance(
    assetId = asset.id.toIdentifier(),
    available = available,
    frozen = BigInteger.ZERO,
    locked = BigInteger.ZERO,
    staked = BigInteger.ZERO,
    pending = BigInteger.ZERO,
    pendingUnconfirmed = BigInteger.ZERO,
    rewards = BigInteger.ZERO,
    reserved = BigInteger.ZERO,
    withdrawable = BigInteger.ZERO,
    earn = BigInteger.ZERO,
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
