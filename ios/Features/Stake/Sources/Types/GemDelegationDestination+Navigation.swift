// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemDelegationDestination
import GemstonePrimitives
import Primitives

extension GemDelegationDestination {
    func navigationValue(delegation: Delegation) -> any Hashable {
        switch self {
        case .details: delegation
        case let .confirm(transfer): transfer
        case let .amount(asset, input): AmountInput(type: input.map(), asset: asset.toPrimitives())
        }
    }
}
