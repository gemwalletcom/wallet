package com.gemwallet.android.testkit

import com.gemwallet.android.domains.perpetual.autoclose.AutocloseError
import com.gemwallet.android.domains.perpetual.autoclose.AutocloseField
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Perpetual
import com.wallet.core.primitives.PerpetualData
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualMarginType
import com.wallet.core.primitives.PerpetualMetadata
import com.wallet.core.primitives.PerpetualPosition
import com.wallet.core.primitives.PerpetualPositionData
import com.wallet.core.primitives.PerpetualProvider
import com.wallet.core.primitives.PerpetualTriggerOrder
import com.wallet.core.primitives.TpslType

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

fun mockPerpetualPosition(
    id: String = "pos-1",
    perpetualId: PerpetualId = PerpetualId(provider = PerpetualProvider.Hypercore, symbol = "TON"),
    assetId: AssetId = mockAsset().id,
    size: Double = 10.0,
    sizeValue: Double = 1000.0,
    leverage: UByte = 5u,
    entryPrice: Double = 100.0,
    liquidationPrice: Double? = 50.0,
    marginType: PerpetualMarginType = PerpetualMarginType.Cross,
    direction: PerpetualDirection = PerpetualDirection.Long,
    marginAmount: Double = 200.0,
    takeProfit: PerpetualTriggerOrder? = null,
    stopLoss: PerpetualTriggerOrder? = null,
    pnl: Double = 0.0,
    funding: Float? = null,
) = PerpetualPosition(
    id = id,
    perpetualId = perpetualId,
    assetId = assetId,
    size = size,
    sizeValue = sizeValue,
    leverage = leverage,
    entryPrice = entryPrice,
    liquidationPrice = liquidationPrice,
    marginType = marginType,
    direction = direction,
    marginAmount = marginAmount,
    takeProfit = takeProfit,
    stopLoss = stopLoss,
    pnl = pnl,
    funding = funding,
)

fun mockPerpetualPositionData(
    perpetual: Perpetual = mockPerpetual(price = 100.0),
    asset: Asset = mockAsset(),
    position: PerpetualPosition = mockPerpetualPosition(assetId = asset.id, perpetualId = perpetual.id),
) = PerpetualPositionData(
    perpetual = perpetual,
    asset = asset,
    position = position,
)

fun mockAutocloseField(
    type: TpslType = TpslType.TakeProfit,
    price: Double? = null,
    originalPrice: Double? = null,
    formattedPrice: String? = price?.toString(),
    error: AutocloseError? = null,
    orderId: ULong? = null,
) = AutocloseField(
    type = type,
    price = price,
    originalPrice = originalPrice,
    formattedPrice = formattedPrice,
    error = error,
    orderId = orderId,
)
