// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation

extension Delegation: Identifiable {
    public var id: String {
        base.id
    }
}

extension DelegationBase: Identifiable {
    public var id: String {
        [assetId.identifier, validatorId, state.rawValue, delegationId].joined(separator: "_")
    }
}

extension DelegationValidator: Identifiable {}

public extension DelegationBase {
    var balanceValue: BigInt {
        BigInt(stringLiteral: balance)
    }

    var rewardsValue: BigInt {
        BigInt(stringLiteral: rewards)
    }

    func with(state: DelegationState) -> DelegationBase {
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

public extension DelegationState {
    init(id: String) throws {
        if let state = DelegationState(rawValue: id) {
            self = state
        } else {
            throw AnyError("invalid state: \(id)")
        }
    }
}
