package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualMarginType
import com.wallet.core.primitives.PerpetualProvider
import uniffi.gemstone.GemPerpetualTransferData
import uniffi.gemstone.PerpetualConfirmData

fun mockPerpetualConfirmData(
    direction: PerpetualDirection = PerpetualDirection.Long,
    fiatValue: Double = 100.0,
    slippage: Double = 2.0,
    leverage: UByte = 1u,
    pnl: Double? = null,
    entryPrice: Double? = null,
    marketPrice: Double = 100.0,
    marginAmount: Double = 100.0,
    takeProfit: String? = null,
    stopLoss: String? = null,
) = PerpetualConfirmData(
    direction = direction.toGem(),
    marginType = PerpetualMarginType.Cross.toGem(),
    baseAsset = mockAssetHyperCoreUSDC().toGem(),
    assetIndex = 0,
    price = "100.0",
    fiatValue = fiatValue,
    size = "1.0",
    slippage = slippage,
    leverage = leverage,
    pnl = pnl,
    entryPrice = entryPrice,
    marketPrice = marketPrice,
    marginAmount = marginAmount,
    takeProfit = takeProfit,
    stopLoss = stopLoss,
)

fun mockGemPerpetualTransferData(direction: PerpetualDirection = PerpetualDirection.Long) = GemPerpetualTransferData(
    provider = PerpetualProvider.Hypercore.toGem(),
    direction = direction.toGem(),
    asset = mockAssetHyperCoreUBTC().toGem(),
    baseAsset = mockAssetHyperCoreUSDC().toGem(),
    assetIndex = 0,
    price = 100.0,
    leverage = 1u,
    marginType = PerpetualMarginType.Cross.toGem(),
)
