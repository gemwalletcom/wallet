// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

extension Delegation: Identifiable {
    public var id: String {
        base.id
    }
}

extension DelegationValidator: Identifiable {}

extension DelegationBase: Identifiable {
    public var id: String {
        [assetId.identifier, validatorId, state.rawValue, delegationId].joined(separator: "_")
    }
}

