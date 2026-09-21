// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import Primitives

public struct DelegationInput: Hashable {
    public let delegation: Delegation
    public let validators: [DelegationValidator]

    public init(delegation: Delegation, validators: [DelegationValidator]) {
        self.delegation = delegation
        self.validators = validators
    }
}

public enum StakeRoute: Hashable {
    case delegation(DelegationInput)
    case transfer(TransferRoute)
}

public typealias StakeRouteAction = ((StakeRoute) -> Void)?
