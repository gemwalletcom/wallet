// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension Gemstone.PerpetualProvider {
    func map() -> Primitives.PerpetualProvider {
        switch self {
        case .hypercore: .hypercore
        }
    }
}

public extension Primitives.PerpetualProvider {
    func map() -> Gemstone.PerpetualProvider {
        switch self {
        case .hypercore: .hypercore
        }
    }
}
