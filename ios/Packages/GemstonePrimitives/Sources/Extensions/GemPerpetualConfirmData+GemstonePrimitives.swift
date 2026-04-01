// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension Gemstone.PerpetualConfirmData {
    func map() throws -> Primitives.PerpetualConfirmData {
        try Primitives.PerpetualConfirmData(
            direction: direction.map(),
            marginType: marginType.map(),
            baseAsset: baseAsset.map(),
            assetIndex: assetIndex,
            price: price,
            fiatValue: fiatValue,
            size: size,
            slippage: slippage,
            leverage: leverage,
            pnl: pnl,
            entryPrice: entryPrice,
            marketPrice: marketPrice,
            marginAmount: marginAmount,
            takeProfit: takeProfit,
            stopLoss: stopLoss,
        )
    }
}

public extension Primitives.PerpetualConfirmData {
    func map() -> Gemstone.PerpetualConfirmData {
        Gemstone.PerpetualConfirmData(
            direction: direction.map(),
            marginType: marginType.map(),
            baseAsset: baseAsset.map(),
            assetIndex: assetIndex,
            price: price,
            fiatValue: fiatValue,
            size: size,
            slippage: slippage,
            leverage: leverage,
            pnl: pnl,
            entryPrice: entryPrice,
            marketPrice: marketPrice,
            marginAmount: marginAmount,
            takeProfit: takeProfit,
            stopLoss: stopLoss,
        )
    }
}
