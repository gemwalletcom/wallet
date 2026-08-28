// Copyright (c). Gem Wallet. All rights reserved.

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
        try StakeValidatorRecord
            .filter(StakeValidatorRecord.Columns.assetId == chain.assetId.identifier)
            .filter(StakeValidatorRecord.Columns.providerType == providerType.rawValue)
            .fetchAll(db)
            .map(\.validator)
    }
}
