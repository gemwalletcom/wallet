// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension GemStakeProviderType {
    func map() -> StakeProviderType {
        switch self {
        case .stake: .stake
        case .earn: .earn
        }
    }
}

public extension StakeProviderType {
    func map() -> GemStakeProviderType {
        switch self {
        case .stake: .stake
        case .earn: .earn
        }
    }
}

public extension GemDelegationValidator {
    func map() throws -> DelegationValidator {
        try DelegationValidator(
            chain: chain.map(),
            id: id,
            name: name,
            isActive: isActive,
            commission: commission,
            apr: apr,
            providerType: providerType.map(),
        )
    }
}

public extension DelegationValidator {
    func map() -> GemDelegationValidator {
        GemDelegationValidator(
            chain: chain.rawValue,
            id: id,
            name: name,
            isActive: isActive,
            commission: commission,
            apr: apr,
            providerType: providerType.map(),
        )
    }
}
