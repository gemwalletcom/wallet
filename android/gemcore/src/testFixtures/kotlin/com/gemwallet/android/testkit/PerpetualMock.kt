package com.gemwallet.android.testkit

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Perpetual
import com.wallet.core.primitives.PerpetualData
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualMetadata
import com.wallet.core.primitives.PerpetualProvider

fun mockPerpetual(
    id: PerpetualId = PerpetualId(provider = PerpetualProvider.Hypercore, symbol = "TON"),
    name: String = "TON",
    provider: PerpetualProvider = PerpetualProvider.Hypercore,
    assetId: AssetId = mockAsset().id,
    identifier: String = "0",
    price: Double = 0.0,
    pricePercentChange24h: Double = 0.0,
    openInterest: Double = 0.0,
    volume24h: Double = 0.0,
    funding: Double = 0.0,
    maxLeverage: UByte = 10u,
    isIsolatedOnly: Boolean = false,
) = Perpetual(
    id = id,
    name = name,
    provider = provider,
    assetId = assetId,
    identifier = identifier,
    price = price,
    pricePercentChange24h = pricePercentChange24h,
    openInterest = openInterest,
    volume24h = volume24h,
    funding = funding,
    maxLeverage = maxLeverage,
    isIsolatedOnly = isIsolatedOnly,
)

fun mockPerpetualData(
    perpetual: Perpetual = mockPerpetual(),
    asset: Asset = mockAsset(),
    metadata: PerpetualMetadata = PerpetualMetadata(isPinned = false),
) = PerpetualData(
    perpetual = perpetual,
    asset = asset,
    metadata = metadata,
)
