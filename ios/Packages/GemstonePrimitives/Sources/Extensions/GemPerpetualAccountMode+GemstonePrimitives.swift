// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension GemPerpetualAccountMode {
    func map() -> PerpetualAccountMode {
        switch self {
        case .standard: .standard
        case .unified: .unified
        }
    }
}

public extension PerpetualAccountMode {
    func map() -> GemPerpetualAccountMode {
        switch self {
        case .standard: .standard
        case .unified: .unified
        }
    }
}
