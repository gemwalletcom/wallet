package com.gemwallet.android.domains.perpetual

import com.gemwallet.android.domains.asset.toGem
import com.wallet.core.primitives.PerpetualConfirmData
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualMarginType
import com.wallet.core.primitives.PerpetualReduceData
import com.wallet.core.primitives.PerpetualType
import uniffi.gemstone.GemPerpetualMarginType
import uniffi.gemstone.PerpetualConfirmData as GemPerpetualConfirmData
import uniffi.gemstone.PerpetualDirection as GemPerpetualDirection
import uniffi.gemstone.PerpetualReduceData as GemPerpetualReduceData
import uniffi.gemstone.PerpetualType as GemPerpetualType

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
    is PerpetualType.Modify -> error("PerpetualType.Modify not produced by Android app")
}
