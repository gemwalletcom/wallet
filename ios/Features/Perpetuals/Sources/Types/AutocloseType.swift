// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Primitives
import PrimitivesComponents

public typealias AutocloseCompletion = (_ takeProfit: InputValidationViewModel, _ stopLoss: InputValidationViewModel) -> Void

public enum AutocloseType {
    case modify(PerpetualPositionData, onTransferAction: TransferDataAction)
    case open(AutocloseOpenData, onComplete: AutocloseCompletion)
}

extension AutocloseType {
    var marketPrice: Double {
        switch self {
        case let .modify(position, _): position.perpetual.price
        case let .open(data, _): data.marketPrice
        }
    }

    var direction: PerpetualDirection {
        switch self {
        case let .modify(position, _): position.position.direction
        case let .open(data, _): data.direction
        }
    }
}
