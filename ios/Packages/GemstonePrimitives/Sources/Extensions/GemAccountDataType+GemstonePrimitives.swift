// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension GemAccountDataType {
    func map() -> AccountDataType {
        switch self {
        case .activate:
            .activate
        }
    }
}

public extension AccountDataType {
    func map() -> GemAccountDataType {
        switch self {
        case .activate:
            .activate
        }
    }
}
