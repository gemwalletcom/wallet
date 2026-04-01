// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension Gemstone.GemEarnType {
    func map() throws -> Primitives.EarnType {
        switch self {
        case let .deposit(validator): try .deposit(validator.map())
        case let .withdraw(delegation): try .withdraw(delegation.map())
        }
    }
}

public extension Primitives.EarnType {
    func map() -> Gemstone.GemEarnType {
        switch self {
        case let .deposit(validator): .deposit(validator.map())
        case let .withdraw(delegation): .withdraw(delegation.map())
        }
    }
}
