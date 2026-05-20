// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension Gemstone.SolanaNftStandard {
    func map() -> Primitives.SolanaNftStandard {
        switch self {
        case .nonFungible: .nonFungible
        case let .programmableNonFungible(ruleSet): .programmableNonFungible(.init(rule_set: ruleSet))
        case let .core(collection): .core(.init(collection: collection))
        }
    }
}

public extension Primitives.SolanaNftStandard {
    func map() -> Gemstone.SolanaNftStandard {
        switch self {
        case .nonFungible: .nonFungible
        case let .programmableNonFungible(inner): .programmableNonFungible(ruleSet: inner.rule_set)
        case let .core(inner): .core(collection: inner.collection)
        }
    }
}
