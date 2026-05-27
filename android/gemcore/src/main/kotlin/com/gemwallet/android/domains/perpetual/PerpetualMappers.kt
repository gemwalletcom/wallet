package com.gemwallet.android.domains.perpetual

import com.gemwallet.android.domains.asset.toGem
import com.wallet.core.primitives.CancelOrderData
import com.wallet.core.primitives.PerpetualConfirmData
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualMarginType
import com.wallet.core.primitives.PerpetualModifyConfirmData
import com.wallet.core.primitives.PerpetualModifyPositionType
import com.wallet.core.primitives.PerpetualReduceData
import com.wallet.core.primitives.PerpetualType
import com.wallet.core.primitives.TPSLOrderData
import uniffi.gemstone.GemPerpetualMarginType
import uniffi.gemstone.CancelOrderData as GemCancelOrderData
import uniffi.gemstone.PerpetualConfirmData as GemPerpetualConfirmData
import uniffi.gemstone.PerpetualDirection as GemPerpetualDirection
import uniffi.gemstone.PerpetualModifyConfirmData as GemPerpetualModifyConfirmData
import uniffi.gemstone.PerpetualModifyPositionType as GemPerpetualModifyPositionType
import uniffi.gemstone.PerpetualReduceData as GemPerpetualReduceData
import uniffi.gemstone.PerpetualType as GemPerpetualType
import uniffi.gemstone.TpslOrderData as GemTpslOrderData

fun PerpetualConfirmData.toGem(): GemPerpetualConfirmData = GemPerpetualConfirmData(
    direction = direction.toGem(),
    marginType = marginType.toGem(),
    baseAsset = baseAsset.toGem(),
    assetIndex = assetIndex,
    price = price,
    fiatValue = fiatValue,
    size = size,
    slippage = slippage,
    leverage = leverage,
    pnl = pnl,
    entryPrice = entryPrice,
    marketPrice = marketPrice,
    marginAmount = marginAmount,
    takeProfit = takeProfit,
    stopLoss = stopLoss,
)

fun PerpetualReduceData.toGem(): GemPerpetualReduceData = GemPerpetualReduceData(
    data = data.toGem(),
    positionDirection = positionDirection.toGem(),
)

fun PerpetualModifyConfirmData.toGem(): GemPerpetualModifyConfirmData = GemPerpetualModifyConfirmData(
    baseAsset = baseAsset.toGem(),
    assetIndex = assetIndex,
    modifyTypes = modifyTypes.map { it.toGem() },
    takeProfitOrderId = takeProfitOrderId?.toULong(),
    stopLossOrderId = stopLossOrderId?.toULong(),
)

fun PerpetualModifyPositionType.toGem(): GemPerpetualModifyPositionType = when (this) {
    is PerpetualModifyPositionType.Tpsl -> GemPerpetualModifyPositionType.Tpsl(v1 = content.toGem())
    is PerpetualModifyPositionType.Cancel -> GemPerpetualModifyPositionType.Cancel(v1 = content.map { it.toGem() })
}

fun TPSLOrderData.toGem(): GemTpslOrderData = GemTpslOrderData(
    direction = direction.toGem(),
    takeProfit = takeProfit,
    stopLoss = stopLoss,
    size = size,
)

fun CancelOrderData.toGem(): GemCancelOrderData = GemCancelOrderData(
    assetIndex = assetIndex,
    orderId = orderId.toULong(),
)

fun PerpetualDirection.toGem(): GemPerpetualDirection = when (this) {
    PerpetualDirection.Long -> GemPerpetualDirection.LONG
    PerpetualDirection.Short -> GemPerpetualDirection.SHORT
}

fun PerpetualMarginType.toGem(): GemPerpetualMarginType = when (this) {
    PerpetualMarginType.Cross -> GemPerpetualMarginType.CROSS
    PerpetualMarginType.Isolated -> GemPerpetualMarginType.ISOLATED
}

fun PerpetualType.toGem(): GemPerpetualType = when (this) {
    is PerpetualType.Open -> GemPerpetualType.Open(v1 = content.toGem())
    is PerpetualType.Close -> GemPerpetualType.Close(v1 = content.toGem())
    is PerpetualType.Increase -> GemPerpetualType.Increase(v1 = content.toGem())
    is PerpetualType.Reduce -> GemPerpetualType.Reduce(v1 = content.toGem())
    is PerpetualType.Modify -> GemPerpetualType.Modify(v1 = content.toGem())
}
