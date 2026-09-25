package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
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
    baseAsset = mockAsset(id = mockAssetId(chain = Chain.HyperCore, tokenId = "USDC::0x6d1e7cde53ba9467b783cb7c530ce054::0"), name = "USDC", symbol = "USDC", decimals = 8, type = AssetType.TOKEN).toGem(),
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
    asset = mockAsset(id = mockAssetId(chain = Chain.HyperCore, tokenId = "UBTC::0x8f254b963e8468305d409b33aa137c67::197"), name = "Bitcoin", symbol = "UBTC", decimals = 10, type = AssetType.TOKEN).toGem(),
    baseAsset = mockAsset(id = mockAssetId(chain = Chain.HyperCore, tokenId = "USDC::0x6d1e7cde53ba9467b783cb7c530ce054::0"), name = "USDC", symbol = "USDC", decimals = 8, type = AssetType.TOKEN).toGem(),
    assetIndex = 0,
    price = 100.0,
    leverage = 1u,
    marginType = PerpetualMarginType.Cross.toGem(),
)
