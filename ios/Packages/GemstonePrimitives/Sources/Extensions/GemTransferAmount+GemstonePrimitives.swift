// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import enum Gemstone.GemTransferAmountError
import Primitives

public extension GemTransferAmountError {
    var requirement: BalanceRequirement {
        switch self {
        case let .InsufficientBalance(_, required, available),
             let .InsufficientNetworkFee(_, required, available),
             let .MinimumAccountBalanceTooLow(_, required, available):
            BalanceRequirement(required: BigInt(core: required), available: BigInt(core: available))
        }
    }

}
