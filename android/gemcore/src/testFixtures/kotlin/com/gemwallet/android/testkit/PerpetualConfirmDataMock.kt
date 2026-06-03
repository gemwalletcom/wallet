package com.gemwallet.android.testkit

import com.gemwallet.android.domains.perpetual.PerpetualTransferData
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.CancelOrderData
import com.wallet.core.primitives.PerpetualConfirmData
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualMarginType
import com.wallet.core.primitives.PerpetualModifyConfirmData
import com.wallet.core.primitives.PerpetualModifyPositionType
import com.wallet.core.primitives.PerpetualProvider
import com.wallet.core.primitives.PerpetualReduceData
import com.wallet.core.primitives.TPSLOrderData

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
    direction = direction,
    marginType = marginType,
    baseAsset = baseAsset,
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
    positionDirection = positionDirection,
)

fun mockPerpetualModifyConfirmData(
    modifyTypes: List<PerpetualModifyPositionType> = emptyList(),
    baseAsset: Asset = mockAssetHyperCoreUSDC(),
    assetIndex: Int = 0,
    takeProfitOrderId: Long? = null,
    stopLossOrderId: Long? = null,
) = PerpetualModifyConfirmData(
    baseAsset = baseAsset,
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
    TPSLOrderData(
        direction = direction,
        takeProfit = takeProfit,
        stopLoss = stopLoss,
        size = size,
    ),
)

fun mockCancel(orderIds: List<Long>, assetIndex: Int = 0) = PerpetualModifyPositionType.Cancel(
    orderIds.map { CancelOrderData(assetIndex = assetIndex, orderId = it) },
)

fun mockPerpetualTransferData(
    provider: PerpetualProvider = PerpetualProvider.Hypercore,
    direction: PerpetualDirection = PerpetualDirection.Long,
    asset: Asset = mockAssetHyperCoreUBTC(),
    baseAsset: Asset = mockAssetHyperCoreUSDC(),
    assetIndex: Int = 0,
    price: Double = 100.0,
    leverage: UByte = 1u,
    marginType: PerpetualMarginType = PerpetualMarginType.Cross,
) = PerpetualTransferData(
    provider = provider,
    direction = direction,
    asset = asset,
    baseAsset = baseAsset,
    assetIndex = assetIndex,
    price = price,
    leverage = leverage,
    marginType = marginType,
)
