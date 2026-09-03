// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemPerpetualPositionAction
import struct Gemstone.GemPerpetualTransferData

public extension GemPerpetualPositionAction {
    var data: GemPerpetualTransferData {
        switch self {
        case let .open(data), let .increase(data), let .reduce(data, _): data
        }
    }
}
