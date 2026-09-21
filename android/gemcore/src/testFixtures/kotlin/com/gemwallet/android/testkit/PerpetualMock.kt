package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Perpetual
import com.wallet.core.primitives.PerpetualData
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualMarginType
import com.wallet.core.primitives.PerpetualMetadata
import com.wallet.core.primitives.PerpetualOrderType
import com.wallet.core.primitives.PerpetualPosition
import com.wallet.core.primitives.PerpetualPositionData
import com.wallet.core.primitives.PerpetualProvider
import com.wallet.core.primitives.PerpetualTriggerOrder
import com.wallet.core.primitives.TpslType
import uniffi.gemstone.AutocloseValidation
import uniffi.gemstone.GemAutocloseField
import uniffi.gemstone.GemAutocloseViewState

fun mockPerpetual(price: Double = 0.0, pricePercentChange24h: Double = 0.0, volume24h: Double = 0.0, funding: Double = 0.0) = Perpetual(
    id = PerpetualId(provider = PerpetualProvider.Hypercore, symbol = "TON"),
    name = "TON",
    provider = PerpetualProvider.Hypercore,
    assetId = mockAsset().id,
    identifier = "0",
    price = price,
    pricePercentChange24h = pricePercentChange24h,
    openInterest = 0.0,
    volume24h = volume24h,
    funding = funding,
    maxLeverage = 10u,
    isIsolatedOnly = false,
)

fun mockPerpetualData(perpetual: Perpetual = mockPerpetual(), asset: Asset = mockAsset()) = PerpetualData(
    perpetual = perpetual,
    asset = asset,
    metadata = PerpetualMetadata(isPinned = false),
)

fun mockPerpetualPosition(
    id: String = "pos-1",
    perpetualId: PerpetualId = PerpetualId(provider = PerpetualProvider.Hypercore, symbol = "TON"),
    assetId: AssetId = mockAsset().id,
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
    size = 10.0,
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

fun mockPerpetualPositionData(perpetual: Perpetual = mockPerpetual(price = 100.0), asset: Asset = mockAsset(), position: PerpetualPosition = mockPerpetualPosition(assetId = asset.id, perpetualId = perpetual.id)) = PerpetualPositionData(
    perpetual = perpetual,
    asset = asset,
    position = position,
)

fun mockPerpetualTriggerOrder(price: Double = 100.0) = PerpetualTriggerOrder(
    price = price,
    order_type = PerpetualOrderType.Limit,
    order_id = "order-$price",
)

fun mockAutocloseField(type: TpslType = TpslType.TakeProfit, price: Double? = null, validation: AutocloseValidation = AutocloseValidation.VALID) = GemAutocloseField(
    tpslType = type.toGem(),
    price = price,
    originalPrice = null,
    formattedPrice = price?.toString(),
    validation = validation,
    orderId = null,
)

fun mockAutocloseViewState(confirmEnabled: Boolean = false, showsErrors: Boolean = false) = GemAutocloseViewState(
    confirmEnabled = confirmEnabled,
    showsErrors = showsErrors,
    entryPrice = mockFormattedNumber(100.0),
    marketPrice = mockFormattedNumber(110.0),
)
