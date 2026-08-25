// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension DelegationBase {
    static func mock(
        state: DelegationState,
        assetId: AssetId = .mock(),
        balance: String = "0",
        shares: String = "0",
        rewards: String = "0",
        completionDate: Date? = nil,
        delegationId: String = .empty,
        validatorId: String = .empty,
    ) -> DelegationBase {
        DelegationBase(
            assetId: assetId,
            state: state,
            balance: balance,
            shares: shares,
            rewards: rewards,
            completionDate: completionDate,
            delegationId: delegationId,
            validatorId: validatorId,
        )
    }
}
