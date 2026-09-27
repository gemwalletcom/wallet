// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import Primitives

public enum StakeRoute: Hashable {
    case delegation(Delegation)
    case transfer(TransferRoute)
}

public typealias StakeRouteAction = ((StakeRoute) -> Void)?
