// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemNftSearchItem
import Primitives

public extension GemNftSearchItem {
    func map() -> NFTSearchItem {
        switch self {
        case let .collection(data): .collection(data.map())
        case let .asset(data): .asset(data.map())
        }
    }
}
