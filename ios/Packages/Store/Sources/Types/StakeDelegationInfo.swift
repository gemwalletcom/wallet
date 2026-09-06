// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

internal import BigInt

struct StakeDelegationInfo: Codable, FetchableRecord {
    let delegation: StakeDelegationRecord
    let validator: StakeValidatorRecord
    let price: PriceRecord?
}

extension StakeDelegationInfo {
    func mapToDelegation() -> Delegation {
        Delegation(
            base: DelegationBase(
                assetId: delegation.assetId,
                state: delegation.state,
                balance: BigInt(stringLiteral: delegation.balance),
                shares: BigInt(stringLiteral: delegation.shares ?? "0"),
                rewards: BigInt(stringLiteral: delegation.rewards),
                completionDate: delegation.completionDate,
                delegationId: delegation.delegationId,
                validatorId: validator.validatorId,
            ),
            validator: validator.validator,
            price: price?.mapToPrice(),
        )
    }
}
