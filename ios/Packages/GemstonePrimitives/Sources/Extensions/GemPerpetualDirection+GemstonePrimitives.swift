// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension Gemstone.PerpetualDirection {
    func map() -> Primitives.PerpetualDirection {
        switch self {
        case .short: .short
        case .long: .long
        }
    }
}

public extension Primitives.PerpetualDirection {
    func map() -> Gemstone.PerpetualDirection {
        switch self {
        case .short: .short
        case .long: .long
        }
    }
}
