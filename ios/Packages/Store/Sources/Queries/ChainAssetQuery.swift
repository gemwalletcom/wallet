// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct ChainAssetQuery: DatabaseQueryable {
    public var assetRequest: AssetQuery
    public var feeAssetRequest: AssetQuery

    public init(walletId: WalletId, assetId: AssetId) {
        assetRequest = AssetQuery(walletId: walletId, assetId: assetId)
        feeAssetRequest = AssetQuery(walletId: walletId, assetId: assetId.chain.assetId)
    }

    public func fetch(_ db: Database) throws -> ChainAssetData {
        if assetRequest.assetId == feeAssetRequest.assetId {
            let assetData = try assetRequest.fetch(db)
            return ChainAssetData(
                assetData: assetData,
                feeAssetData: assetData,
            )
        }
        return try ChainAssetData(
            assetData: assetRequest.fetch(db),
            feeAssetData: feeAssetRequest.fetch(db),
        )
    }
}

extension ChainAssetQuery: Equatable {}
