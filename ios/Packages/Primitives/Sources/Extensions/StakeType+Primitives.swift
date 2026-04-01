// Copyright (c). Gem Wallet. All rights reserved.

public extension StakeType {
    var validatorId: String {
        switch self {
        case let .stake(validator): validator.id
        case let .unstake(delegation): delegation.validator.id
        case let .redelegate(data): data.delegation.validator.id
        case let .rewards(validators): validators.first?.id ?? .empty
        case let .withdraw(delegation): delegation.validator.id
        case .freeze, .unfreeze: .empty
        }
    }
}
