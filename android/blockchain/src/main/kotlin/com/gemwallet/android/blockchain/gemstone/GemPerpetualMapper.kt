package com.gemwallet.android.blockchain.gemstone

import com.gemwallet.android.domains.asset.toDTO
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toPerpetualId
import com.wallet.core.primitives.ChartCandleStick
import com.wallet.core.primitives.Perpetual
import com.wallet.core.primitives.PerpetualBalance
import com.wallet.core.primitives.PerpetualData
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualMarginType
import com.wallet.core.primitives.PerpetualMetadata
import com.wallet.core.primitives.PerpetualOrderType
import com.wallet.core.primitives.PerpetualPosition
import com.wallet.core.primitives.PerpetualPositionsSummary
import com.wallet.core.primitives.PerpetualProvider
import com.wallet.core.primitives.PerpetualTriggerOrder
import uniffi.gemstone.GemChartCandleStick
import uniffi.gemstone.GemPerpetualData
import uniffi.gemstone.GemPerpetualMarginType
import uniffi.gemstone.GemPerpetualOrderType
import uniffi.gemstone.GemPerpetualPosition
import uniffi.gemstone.GemPerpetualPositionsSummary
import uniffi.gemstone.GemPerpetualTriggerOrder

internal fun GemPerpetualData.toDTO(): PerpetualData? {
    return PerpetualData(
        perpetual = Perpetual(
            id = perpetual.id.toPerpetualId() ?: return null,
            name = perpetual.name,
            provider = perpetual.provider.toDTO(),
            assetId = perpetual.assetId.toAssetId() ?: return null,
            identifier = perpetual.identifier,
            price = perpetual.price,
            pricePercentChange24h = perpetual.pricePercentChange24h,
            openInterest = perpetual.openInterest,
            volume24h = perpetual.volume24h,
            funding = perpetual.funding,
            maxLeverage = perpetual.maxLeverage,
            isIsolatedOnly = perpetual.isIsolatedOnly,
        ),
        asset = asset.toDTO(),
        metadata = PerpetualMetadata(
            isPinned = metadata.isPinned,
        ),
    )
}

internal fun GemPerpetualPositionsSummary.toDTO(): PerpetualPositionsSummary {
    return PerpetualPositionsSummary(
        positions = positions.mapNotNull { it.toDTO() },
        balance = PerpetualBalance(
            available = balance.available,
            reserved = balance.reserved,
            withdrawable = balance.withdrawable,
        ),
    )
}

internal fun GemPerpetualPosition.toDTO(): PerpetualPosition? {
    return PerpetualPosition(
        id = id,
        perpetualId = perpetualId.toPerpetualId() ?: return null,
        assetId = assetId.toAssetId() ?: return null,
        size = size,
        sizeValue = sizeValue,
        leverage = leverage,
        entryPrice = entryPrice,
        liquidationPrice = liquidationPrice,
        marginType = when (marginType) {
            GemPerpetualMarginType.CROSS -> PerpetualMarginType.Cross
            GemPerpetualMarginType.ISOLATED -> PerpetualMarginType.Isolated
        },
        direction = when (direction) {
            uniffi.gemstone.PerpetualDirection.SHORT -> PerpetualDirection.Short
            uniffi.gemstone.PerpetualDirection.LONG -> PerpetualDirection.Long
        },
        marginAmount = marginAmount,
        takeProfit = takeProfit?.let {
            it.toDTO()
        },
        stopLoss = stopLoss?.let {
            it.toDTO()
        },
        pnl = pnl,
        funding = funding,
    )
}

internal fun GemChartCandleStick.toDTO(): ChartCandleStick {
    return ChartCandleStick(
        date = date,
        open = open,
        high = high,
        low = low,
        close = close,
        volume = volume,
    )
}

private fun GemPerpetualTriggerOrder.toDTO(): PerpetualTriggerOrder {
    return PerpetualTriggerOrder(
        price = price,
        order_type = when (orderType) {
            GemPerpetualOrderType.MARKET -> PerpetualOrderType.Market
            GemPerpetualOrderType.LIMIT -> PerpetualOrderType.Limit
        },
        order_id = orderId,
    )
}

private fun uniffi.gemstone.PerpetualProvider.toDTO(): PerpetualProvider = when (this) {
    uniffi.gemstone.PerpetualProvider.HYPERCORE -> PerpetualProvider.Hypercore
}
