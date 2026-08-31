// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension Gemstone.FeePriority {
    func map() -> Primitives.FeePriority {
        switch self {
        case .normal: .normal
        case .fast: .fast
        }
    }
}

public extension Primitives.FeePriority {
    func map() -> Gemstone.FeePriority {
        switch self {
        case .normal: .normal
        case .fast: .fast
        }
    }
}
