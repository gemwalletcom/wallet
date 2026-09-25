package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemAssetBalance
import uniffi.gemstone.GemConfirmMetadata
import java.math.BigInteger

fun mockGemAssetBalance(asset: Asset = mockAsset(id = mockAssetId(chain = Chain.Ethereum), name = "Ethereum", symbol = "ETH", decimals = 18), available: BigInteger = BigInteger.ZERO) = GemAssetBalance(
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
    isActive = true,
)

fun mockGemConfirmMetadata(asset: Asset = mockAsset(id = mockAssetId(chain = Chain.Ethereum), name = "Ethereum", symbol = "ETH", decimals = 18)) = GemConfirmMetadata(
    assetBalance = mockGemAssetBalance(asset),
    feeAssetBalance = mockGemAssetBalance(asset),
    prices = emptyList(),
)
