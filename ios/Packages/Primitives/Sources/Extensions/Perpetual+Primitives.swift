// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public extension PerpetualMarginType {
    init(id: String) throws {
        if let type = PerpetualMarginType(rawValue: id) {
            self = type
        } else {
            throw AnyError("invalid margin type: \(id)")
        }
    }
}

public extension PerpetualDirection {
    init(id: String) throws {
        if let direction = PerpetualDirection(rawValue: id) {
            self = direction
        } else {
            throw AnyError("invalid direction: \(id)")
        }
    }
}


