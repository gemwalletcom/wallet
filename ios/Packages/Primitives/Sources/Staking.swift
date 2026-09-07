// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct RedelegateData: Codable, Equatable, Hashable, Sendable {
    public let delegation: Delegation
    public let toValidator: DelegationValidator

    public init(delegation: Delegation, toValidator: DelegationValidator) {
        self.delegation = delegation
        self.toValidator = toValidator
    }
}

public enum StakeType: Codable, Equatable, Hashable, Sendable {
    case stake(DelegationValidator)
    case unstake(Delegation)
    case redelegate(RedelegateData)
    case rewards([DelegationValidator])
    case withdraw(Delegation)
    case freeze(Resource)
    case unfreeze(Resource)
}
