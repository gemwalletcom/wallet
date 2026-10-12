// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct PerpetualResult: Equatable, Sendable {
    public let data: PerpetualData
    public let associatedAssets: [Asset]

    public init(data: PerpetualData, associatedAssets: [Asset]) {
        self.data = data
        self.associatedAssets = associatedAssets
    }

    public static let empty = PerpetualResult(data: .empty, associatedAssets: [])
}

public struct PerpetualQuery: DatabaseQueryable {
    public let assetId: AssetId

    public init(assetId: AssetId) {
        self.assetId = assetId
    }

    public func fetch(_ db: Database) throws -> PerpetualResult {
        guard let info = try PerpetualRecord
            .including(required: PerpetualRecord.asset)
            .filter(PerpetualRecord.Columns.assetId == assetId.identifier)
            .asRequest(of: PerpetualInfo.self)
            .fetchOne(db)
        else {
            return .empty
        }
        let assetIds = info.asset.associations.filter { $0.type == .official }.map(\.assetId.identifier)
        let associatedAssets = try AssetRecord
            .filter(assetIds.contains(AssetRecord.Columns.id))
            .fetchAll(db)
            .map { $0.mapToAsset() }
        return PerpetualResult(data: info.mapToPerpetualData(), associatedAssets: associatedAssets)
    }
}

// MARK: - Models Extensions

extension PerpetualQuery: Equatable {}

public extension PerpetualData {
    static var empty: PerpetualData {
        PerpetualData(
            perpetual: Perpetual(
                id: PerpetualId(provider: .hypercore, symbol: ""),
                name: "",
                provider: .hypercore,
                assetId: AssetId(chain: .bitcoin, tokenId: nil),
                identifier: "",
                price: .zero,
                pricePercentChange24h: .zero,
                openInterest: .zero,
                volume24h: .zero,
                funding: .zero,
                maxLeverage: 1,
                isIsolatedOnly: false,
            ),
            asset: Asset(id: .init(chain: .bitcoin, tokenId: .none), name: "", symbol: "", decimals: 0, type: .native),
            metadata: PerpetualMetadata(isPinned: false),
        )
    }
}
