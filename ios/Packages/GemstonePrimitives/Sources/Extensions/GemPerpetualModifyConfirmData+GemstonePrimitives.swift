// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension Gemstone.PerpetualModifyConfirmData {
    func map() throws -> Primitives.PerpetualModifyConfirmData {
        try Primitives.PerpetualModifyConfirmData(
            baseAsset: baseAsset.map(),
            assetIndex: assetIndex,
            modifyTypes: modifyTypes.map { try $0.map() },
            takeProfitOrderId: takeProfitOrderId,
            stopLossOrderId: stopLossOrderId,
        )
    }
}

public extension Primitives.PerpetualModifyConfirmData {
    func map() -> Gemstone.PerpetualModifyConfirmData {
        Gemstone.PerpetualModifyConfirmData(
            baseAsset: baseAsset.map(),
            assetIndex: assetIndex,
            modifyTypes: modifyTypes.map { $0.map() },
            takeProfitOrderId: takeProfitOrderId,
            stopLossOrderId: stopLossOrderId,
        )
    }
}
