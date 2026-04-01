// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension Gemstone.PerpetualModifyPositionType {
    func map() throws -> Primitives.PerpetualModifyPositionType {
        switch self {
        case let .tpsl(data):
            try .tpsl(data.map())
        case let .cancel(orders):
            try .cancel(orders.map { try $0.map() })
        }
    }
}

public extension Primitives.PerpetualModifyPositionType {
    func map() -> Gemstone.PerpetualModifyPositionType {
        switch self {
        case let .tpsl(data):
            .tpsl(data.map())
        case let .cancel(orders):
            .cancel(orders.map { $0.map() })
        }
    }
}
