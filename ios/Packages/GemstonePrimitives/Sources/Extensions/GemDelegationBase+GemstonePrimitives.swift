// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension GemDelegationState {
    func map() -> DelegationState {
        switch self {
        case .active: .active
        case .pending: .pending
        case .inactive: .inactive
        case .activating: .activating
        case .deactivating: .deactivating
        case .awaitingWithdrawal: .awaitingWithdrawal
        }
    }
}

public extension DelegationState {
    func map() -> GemDelegationState {
        switch self {
        case .active: .active
        case .pending: .pending
        case .inactive: .inactive
        case .activating: .activating
        case .deactivating: .deactivating
        case .awaitingWithdrawal: .awaitingWithdrawal
        }
    }
}

public extension GemDelegationBase {
    func map() throws -> DelegationBase {
        try DelegationBase(
            assetId: AssetId(id: assetId),
            state: state.map(),
            balance: balance,
            shares: shares,
            rewards: rewards,
            completionDate: completionDate.map { Date(timeIntervalSince1970: TimeInterval($0)) },
            delegationId: delegationId,
            validatorId: validatorId,
        )
    }
}

public extension DelegationBase {
    func map() -> GemDelegationBase {
        GemDelegationBase(
            assetId: assetId.identifier,
            state: state.map(),
            balance: balance,
            shares: shares,
            rewards: rewards,
            completionDate: completionDate.map { Int64($0.timeIntervalSince1970) },
            delegationId: delegationId,
            validatorId: validatorId,
        )
    }
}
