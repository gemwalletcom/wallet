// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import Primitives

public extension GemResource {
    func map() -> Primitives.Resource {
        switch self {
        case .bandwidth: .bandwidth
        case .energy: .energy
        }
    }
}

public extension Primitives.Resource {
    func map() -> GemResource {
        switch self {
        case .bandwidth: .bandwidth
        case .energy: .energy
        }
    }
}

public extension GemStakeType {
    func map() throws -> StakeType {
        switch self {
        case let .delegate(validator):
            return try .stake(validator.map())
        case let .undelegate(delegation):
            return try .unstake(delegation.map())
        case let .redelegate(delegation, toValidator):
            return try .redelegate(RedelegateData(delegation: delegation.map(), toValidator: toValidator.map()))
        case let .withdrawRewards(validators):
            let mappedValidators = try validators.map { try $0.map() }
            return .rewards(mappedValidators)
        case let .withdraw(delegation):
            return try .withdraw(delegation.map())
        case let .freeze(resource):
            return .freeze(resource.map())
        case let .unfreeze(resource):
            return .unfreeze(resource.map())
        }
    }
}

public extension StakeType {
    func map() -> GemStakeType {
        switch self {
        case let .stake(validator):
            .delegate(validator: validator.map())
        case let .unstake(delegation):
            .undelegate(delegation: delegation.map())
        case let .redelegate(data):
            .redelegate(delegation: data.delegation.map(), toValidator: data.toValidator.map())
        case let .rewards(validators):
            .withdrawRewards(validators: validators.map { $0.map() })
        case let .withdraw(delegation):
            .withdraw(delegation: delegation.map())
        case let .freeze(resource):
            .freeze(resource: resource.map())
        case let .unfreeze(resource):
            .unfreeze(resource: resource.map())
        }
    }
}
