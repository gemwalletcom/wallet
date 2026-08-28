package com.gemwallet.android.domains.perpetual

import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Perpetual
import com.wallet.core.primitives.PerpetualConfirmData
import com.wallet.core.primitives.PerpetualPosition
import com.wallet.core.primitives.PerpetualType
import com.gemwallet.android.ext.PerpetualFormatter.toGemProvider
import uniffi.gemstone.GemPerpetualCloseInput
import uniffi.gemstone.GemPerpetualOrderAction
import uniffi.gemstone.GemPerpetualOrderInput
import uniffi.gemstone.perpetualCloseOrder
import uniffi.gemstone.perpetualOrder
import java.math.BigInteger

object PerpetualOrderFactory {

    fun makePerpetualOrder(
        positionAction: PerpetualPositionAction,
        usdcAmount: BigInteger,
        usdcDecimals: Int,
        leverage: UByte,
        takeProfit: String? = null,
        stopLoss: String? = null,
    ): PerpetualType {
        val data = positionAction.data
        val input = GemPerpetualOrderInput(
            action = positionAction.orderAction(),
            direction = data.direction.toJson(),
            marginType = data.marginType.toJson(),
            baseAsset = data.baseAsset.toJson(),
            asset = data.asset.toJson(),
            assetIndex = data.assetIndex,
            provider = data.provider.toGemProvider(),
            price = data.price,
            usdcAmount = usdcAmount.toString(),
            usdcDecimals = usdcDecimals,
            leverage = leverage,
            slippage = null,
            takeProfit = takeProfit,
            stopLoss = stopLoss,
        )
        return perpetualOrder(input).decodeJson()
    }

    fun makeCloseOrder(
        assetIndex: Int,
        perpetual: Perpetual,
        position: PerpetualPosition,
        asset: Asset,
        baseAsset: Asset,
    ): PerpetualConfirmData {
        val input = GemPerpetualCloseInput(
            assetIndex = assetIndex,
            direction = position.direction.toJson(),
            marginType = position.marginType.toJson(),
            baseAsset = baseAsset.toJson(),
            asset = asset.toJson(),
            provider = perpetual.provider.toGemProvider(),
            marketPrice = perpetual.price,
            size = position.size,
            leverage = position.leverage,
            pnl = position.pnl,
            entryPrice = position.entryPrice,
            marginAmount = position.marginAmount,
            slippage = null,
        )
        return perpetualCloseOrder(input).decodeJson()
    }

    private fun PerpetualPositionAction.orderAction(): GemPerpetualOrderAction = when (this) {
        is PerpetualPositionAction.Open -> GemPerpetualOrderAction.Open
        is PerpetualPositionAction.Increase -> GemPerpetualOrderAction.Increase
        is PerpetualPositionAction.Reduce -> GemPerpetualOrderAction.Reduce(positionDirection.toJson())
    }
}
