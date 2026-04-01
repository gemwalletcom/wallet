// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension GemPerpetualPosition {
    func map() throws -> PerpetualPosition {
        try PerpetualPosition(
            id: id,
            perpetualId: perpetualId,
            assetId: AssetId(id: assetId),
            size: size,
            sizeValue: sizeValue,
            leverage: leverage,
            entryPrice: entryPrice,
            liquidationPrice: liquidationPrice,
            marginType: marginType.map(),
            direction: direction.map(),
            marginAmount: marginAmount,
            takeProfit: takeProfit?.map(),
            stopLoss: stopLoss?.map(),
            pnl: pnl,
            funding: funding,
        )
    }
}

public extension PerpetualPosition {
    func map() -> GemPerpetualPosition {
        GemPerpetualPosition(
            id: id,
            perpetualId: perpetualId,
            assetId: assetId.identifier,
            size: size,
            sizeValue: sizeValue,
            leverage: leverage,
            entryPrice: entryPrice,
            liquidationPrice: liquidationPrice,
            marginType: marginType.map(),
            direction: direction.map(),
            marginAmount: marginAmount,
            takeProfit: takeProfit?.map(),
            stopLoss: stopLoss?.map(),
            pnl: pnl,
            funding: funding,
        )
    }
}
