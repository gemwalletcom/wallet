// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemDelegationDestination
import GemstonePrimitives
import Primitives

extension GemDelegationDestination {
    func route(delegation: Delegation) -> StakeRoute {
        switch self {
        case .details: .delegation(delegation)
        case let .confirm(transfer): .transfer(.confirm(transfer))
        case let .amount(asset, request): .transfer(.amount(AmountInput(request: request, asset: asset.toPrimitives())))
        }
    }
}
