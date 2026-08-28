// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.stakeSelectableValidators
import GemstonePrimitives
import GRDB
import Primitives

public struct ValidatorsRequest: DatabaseQueryable {
    private let chain: Chain
    private let providerType: StakeProviderType

    public init(chain: Chain, providerType: StakeProviderType) {
        self.chain = chain
        self.providerType = providerType
    }

    public func fetch(_ db: Database) throws -> [DelegationValidator] {
        let validators = try StakeValidatorRecord
            .filter(StakeValidatorRecord.Columns.assetId == chain.assetId.identifier)
            .filter(StakeValidatorRecord.Columns.providerType == providerType.rawValue)
            .fetchAll(db)
            .map(\.validator)
        return try stakeSelectableValidators(validators: validators.map { try $0.json() }).map { try DelegationValidator($0) }
    }
}
