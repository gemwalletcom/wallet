// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemDelegationAmountInput

public extension GemDelegationAmountInput {
    func map() -> AmountType {
        switch self {
        case let .stake(input): .stake(input)
        case let .earn(earnType): .earn(earnType)
        }
    }
}
