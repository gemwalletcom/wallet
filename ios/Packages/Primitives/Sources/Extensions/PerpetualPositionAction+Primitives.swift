// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation

public extension PerpetualPositionAction {
    var transferData: PerpetualTransferData {
        switch self {
        case let .open(data): data
        case let .reduce(data, _, _): data
        case let .increase(data): data
        }
    }
}
