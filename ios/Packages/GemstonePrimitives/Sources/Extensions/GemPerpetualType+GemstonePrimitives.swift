// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension Gemstone.PerpetualType {
    func map() throws -> Primitives.PerpetualType {
        switch self {
        case let .open(confirmData): try .open(confirmData.map())
        case let .close(confirmData): try .close(confirmData.map())
        case let .increase(confirmData): try .increase(confirmData.map())
        case let .reduce(reduceData): try .reduce(reduceData.map())
        case let .modify(data): try .modify(data.map())
        }
    }
}

public extension Primitives.PerpetualType {
    func map() -> Gemstone.PerpetualType {
        switch self {
        case let .open(data), let .increase(data): .open(data.map())
        case let .reduce(data): .open(data.data.map())
        case let .close(data): .close(data.map())
        case let .modify(data): .modify(data.map())
        }
    }
}
