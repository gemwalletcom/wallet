// Copyright (c). Gem Wallet. All rights reserved.

internal import BigInt
import Foundation
import GRDB
import Primitives

public struct DelegationsRequest: DatabaseQueryable {
    private let walletId: WalletId
    private let assetId: AssetId
    private let providerType: StakeProviderType

    public init(walletId: WalletId, assetId: AssetId, providerType: StakeProviderType) {
        self.walletId = walletId
        self.assetId = assetId
        self.providerType = providerType
    }

    public func fetch(_ db: Database) throws -> [Delegation] {
        try StakeDelegationRecord
            .including(optional: StakeDelegationRecord.validator)
            .including(optional: StakeDelegationRecord.price)
            .filter(StakeDelegationRecord.Columns.walletId == walletId.id)
            .filter(StakeDelegationRecord.Columns.assetId == assetId.identifier)
            .joining(required: StakeDelegationRecord.validator
                .filter(StakeValidatorRecord.Columns.providerType == providerType.rawValue))
            .asRequest(of: StakeDelegationInfo.self)
            .fetchAll(db)
            .compactMap { $0.mapToDelegation() }
            .sorted { $0.base.balanceValue > $1.base.balanceValue }
    }
}
