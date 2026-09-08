package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualMarginType
import com.wallet.core.primitives.PerpetualProvider
import uniffi.gemstone.CancelOrderData
import uniffi.gemstone.GemPerpetualTransferData
import uniffi.gemstone.PerpetualConfirmData
import uniffi.gemstone.PerpetualModifyConfirmData
import uniffi.gemstone.PerpetualModifyPositionType
import uniffi.gemstone.PerpetualReduceData
import uniffi.gemstone.TpslOrderData

fun mockPerpetualConfirmData(
    direction: PerpetualDirection = PerpetualDirection.Long,
    marginType: PerpetualMarginType = PerpetualMarginType.Cross,
    baseAsset: Asset = mockAssetHyperCoreUSDC(),
    assetIndex: Int = 0,
    price: String = "100.0",
    fiatValue: Double = 100.0,
    size: String = "1.0",
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

fun mockPerpetualReduceData(
    data: PerpetualConfirmData = mockPerpetualConfirmData(),
    positionDirection: PerpetualDirection = PerpetualDirection.Long,
) = PerpetualReduceData(
    data = data,
    positionDirection = positionDirection.toGem(),
)

fun mockPerpetualModifyConfirmData(
    modifyTypes: List<PerpetualModifyPositionType> = emptyList(),
    baseAsset: Asset = mockAssetHyperCoreUSDC(),
    assetIndex: Int = 0,
    takeProfitOrderId: ULong? = null,
    stopLossOrderId: ULong? = null,
) = PerpetualModifyConfirmData(
    baseAsset = baseAsset.toGem(),
    assetIndex = assetIndex,
    modifyTypes = modifyTypes,
    takeProfitOrderId = takeProfitOrderId,
    stopLossOrderId = stopLossOrderId,
)

fun mockTpslOrder(
    direction: PerpetualDirection = PerpetualDirection.Long,
    takeProfit: String? = null,
    stopLoss: String? = null,
    size: String = "1.0",
) = PerpetualModifyPositionType.Tpsl(
    TpslOrderData(
        direction = direction.toGem(),
        takeProfit = takeProfit,
        stopLoss = stopLoss,
        size = size,
    ),
)

fun mockCancel(orderIds: List<ULong>, assetIndex: Int = 0) = PerpetualModifyPositionType.Cancel(
    orderIds.map { CancelOrderData(assetIndex = assetIndex, orderId = it) },
)

fun mockGemPerpetualTransferData(
    direction: PerpetualDirection = PerpetualDirection.Long,
    asset: Asset = mockAssetHyperCoreUBTC(),
    leverage: UByte = 1u,
) = GemPerpetualTransferData(
    provider = PerpetualProvider.Hypercore.toGem(),
    direction = direction.toGem(),
    asset = asset.toGem(),
    baseAsset = mockAssetHyperCoreUSDC().toGem(),
    assetIndex = 0,
    price = 100.0,
    leverage = leverage,
    marginType = PerpetualMarginType.Cross.toGem(),
)
