// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import enum Gemstone.GemConfirmError
import Primitives

public extension GemConfirmError {
    var balanceRequirement: BalanceRequirement? {
        switch self {
        case let .InsufficientBalance(_, required, available),
             let .MinimumAccountBalanceTooLow(_, required, available):
            BalanceRequirement(required: BigInt(core: required), available: BigInt(core: available))
        case let .InsufficientNetworkFee(_, required?, available?):
            BalanceRequirement(required: BigInt(core: required), available: BigInt(core: available))
        default: nil
        }
    }
}
