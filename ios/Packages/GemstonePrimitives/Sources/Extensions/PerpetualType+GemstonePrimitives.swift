// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.PerpetualType
import Primitives

public extension PerpetualType {
    var baseAsset: Primitives.Asset {
        switch self {
        case let .open(data), let .close(data), let .increase(data): data.baseAsset.toPrimitives()
        case let .modify(data): data.baseAsset.toPrimitives()
        case let .reduce(reduceData): reduceData.data.baseAsset.toPrimitives()
        }
    }
}
