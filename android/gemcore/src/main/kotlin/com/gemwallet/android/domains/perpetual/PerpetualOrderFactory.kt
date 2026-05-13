package com.gemwallet.android.domains.perpetual

import com.gemwallet.android.ext.PerpetualFormatter
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Perpetual
import com.wallet.core.primitives.PerpetualConfirmData
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualMarginType
import com.wallet.core.primitives.PerpetualPosition
import com.wallet.core.primitives.PerpetualProvider
import com.wallet.core.primitives.PerpetualReduceData
import com.wallet.core.primitives.PerpetualType
import java.math.BigInteger
import kotlin.math.abs
import kotlin.math.pow

object PerpetualOrderFactory {

    private const val DEFAULT_SLIPPAGE_PERCENT: Double = 2.0

    fun makePerpetualOrder(
        positionAction: PerpetualPositionAction,
        usdcAmount: BigInteger,
        usdcDecimals: Int,
        leverage: UByte,
        slippage: Double = DEFAULT_SLIPPAGE_PERCENT,
        takeProfit: String? = null,
        stopLoss: String? = null,
    ): PerpetualType {
        val data = positionAction.data
        val slippagePrice = calculateSlippagePrice(
            marketPrice = data.price,
            direction = data.direction,
            action = OrderAction.Open,
            slippage = slippage,
        )
        val usdAmount = usdcAmount.toDouble() / 10.0.pow(usdcDecimals)
        val sizeAsAsset = (usdAmount * leverage.toDouble()) / data.price
        val fiatValue = data.price * sizeAsAsset
        val marginAmount = fiatValue / leverage.toDouble()
        val confirmData = makePerpetualConfirmData(
            direction = data.direction,
            marginType = data.marginType,
            baseAsset = data.baseAsset,
            fiatValue = fiatValue,
            assetIndex = data.assetIndex,
            provider = data.provider,
            slippagePrice = slippagePrice,
            sizeAsDouble = sizeAsAsset,
            assetDecimals = data.asset.decimals,
            slippage = slippage,
            leverage = leverage,
            pnl = null,
            entryPrice = null,
            marketPrice = data.price,
            marginAmount = marginAmount,
            takeProfit = takeProfit,
            stopLoss = stopLoss,
        )
        return when (positionAction) {
            is PerpetualPositionAction.Open -> PerpetualType.Open(confirmData)
            is PerpetualPositionAction.Increase -> PerpetualType.Increase(confirmData)
            is PerpetualPositionAction.Reduce -> PerpetualType.Reduce(
                PerpetualReduceData(data = confirmData, positionDirection = positionAction.positionDirection),
            )
        }
    }

    fun makeCloseOrder(
        assetIndex: Int,
        perpetual: Perpetual,
        position: PerpetualPosition,
        asset: Asset,
        baseAsset: Asset,
        slippage: Double = DEFAULT_SLIPPAGE_PERCENT,
    ): PerpetualConfirmData {
        val positionPrice = calculateSlippagePrice(
            marketPrice = perpetual.price,
            direction = position.direction,
            action = OrderAction.Close,
            slippage = slippage,
        )
        val absSize = abs(position.size)
        return makePerpetualConfirmData(
            direction = position.direction,
            marginType = position.marginType,
            baseAsset = baseAsset,
            fiatValue = absSize * positionPrice,
            assetIndex = assetIndex,
            provider = perpetual.provider,
            slippagePrice = positionPrice,
            sizeAsDouble = absSize,
            assetDecimals = asset.decimals,
            slippage = slippage,
            leverage = position.leverage,
            pnl = position.pnl,
            entryPrice = position.entryPrice,
            marketPrice = perpetual.price,
            marginAmount = position.marginAmount,
        )
    }

    internal fun calculateSlippagePrice(
        marketPrice: Double,
        direction: PerpetualDirection,
        action: OrderAction,
        slippage: Double,
    ): Double {
        val fraction = slippage / 100.0
        val multiplier = when {
            direction == PerpetualDirection.Long && action == OrderAction.Open -> 1.0 + fraction
            direction == PerpetualDirection.Short && action == OrderAction.Close -> 1.0 + fraction
            direction == PerpetualDirection.Long && action == OrderAction.Close -> 1.0 - fraction
            direction == PerpetualDirection.Short && action == OrderAction.Open -> 1.0 - fraction
            else -> 1.0
        }
        return marketPrice * multiplier
    }

    internal enum class OrderAction { Open, Close }

    private fun makePerpetualConfirmData(
        direction: PerpetualDirection,
        marginType: PerpetualMarginType,
        baseAsset: Asset,
        fiatValue: Double,
        assetIndex: Int,
        provider: PerpetualProvider,
        slippagePrice: Double,
        sizeAsDouble: Double,
        assetDecimals: Int,
        slippage: Double,
        leverage: UByte,
        pnl: Double?,
        entryPrice: Double?,
        marketPrice: Double,
        marginAmount: Double,
        takeProfit: String? = null,
        stopLoss: String? = null,
    ): PerpetualConfirmData = PerpetualConfirmData(
        direction = direction,
        marginType = marginType,
        baseAsset = baseAsset,
        assetIndex = assetIndex,
        price = PerpetualFormatter.formatPrice(provider, slippagePrice, assetDecimals),
        fiatValue = fiatValue,
        size = PerpetualFormatter.formatSize(provider, sizeAsDouble, assetDecimals),
        slippage = slippage,
        leverage = leverage,
        pnl = pnl,
        entryPrice = entryPrice,
        marketPrice = marketPrice,
        marginAmount = marginAmount,
        takeProfit = takeProfit,
        stopLoss = stopLoss,
    )
}
