// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct AssetStore: Sendable {
    let db: DatabaseQueue

    public init(db: DB) {
        self.db = db.dbQueue
    }

    public func insert(assets: [AssetBasic]) throws {
        try db.write { db in
            for asset in assets {
                try asset.record.insert(db, onConflict: .ignore)
            }
        }
    }

    public func add(assets: [AssetBasic]) throws {
        try db.write { db in
            for asset in assets {
                try asset.asset.record.insert(db, onConflict: .ignore)
                try AssetRecord
                    .filter(AssetRecord.Columns.id == asset.asset.id.identifier)
                    .updateAll(
                        db,
                        AssetRecord.Columns.rank.set(to: asset.score.rank.asInt),
                        AssetRecord.Columns.name.set(to: asset.asset.name),
                        AssetRecord.Columns.symbol.set(to: asset.asset.symbol),
                        AssetRecord.Columns.decimals.set(to: asset.asset.decimals),
                        AssetRecord.Columns.type.set(to: asset.asset.type.rawValue),
                        AssetRecord.Columns.isEnabled.set(to: asset.properties.isEnabled),
                        AssetRecord.Columns.isBuyable.set(to: asset.properties.isBuyable),
                        AssetRecord.Columns.isSellable.set(to: asset.properties.isSellable),
                        AssetRecord.Columns.isSwappable.set(to: asset.properties.isSwapable),
                        AssetRecord.Columns.isStakeable.set(to: asset.properties.isStakeable),
                        AssetRecord.Columns.isEarnable.set(to: asset.properties.isEarnable),
                        AssetRecord.Columns.stakingApr.set(to: asset.properties.stakingApr),
                        AssetRecord.Columns.earnApr.set(to: asset.properties.earnApr),
                    )
            }
        }
    }

    public func getAssetsData(walletId: WalletId, filters: [AssetsRequestFilter], limit: Int? = AssetsRequest.defaultQueryLimit) throws -> [AssetData] {
        try db.read { db in
            try AssetsRequest(walletId: walletId, filters: filters, limit: limit).fetch(db)
        }
    }

    public func getAssetData(walletId: WalletId, assetId: AssetId) throws -> AssetData {
        try db.read { db in
            try AssetRequest(walletId: walletId, assetId: assetId).fetch(db)
        }
    }

    public func getAssets() throws -> [Asset] {
        try db.read { db in
            try AssetRecord
                .fetchAll(db)
                .map { $0.mapToAsset() }
        }
    }

    public func getAssets(for assetIds: [String]) throws -> [Asset] {
        try db.read { db in
            try AssetRecord
                .filter(assetIds.contains(AssetRecord.Columns.id))
                .fetchAll(db)
                .map { $0.mapToAsset() }
        }
    }

    @discardableResult
    public func setAssetIsBuyable(for assetIds: [String], value: Bool) throws -> Int {
        try setColumn(for: assetIds, column: AssetRecord.Columns.isBuyable, value: value)
    }

    @discardableResult
    public func setAssetIsSwappable(for assetIds: [String], value: Bool) throws -> Int {
        try setColumn(for: assetIds, column: AssetRecord.Columns.isSwappable, value: value)
    }

    @discardableResult
    public func setAssetIsStakeable(for assetIds: [String], value: Bool) throws -> Int {
        try setColumn(for: assetIds, column: AssetRecord.Columns.isStakeable, value: value)
    }

    @discardableResult
    public func updateBuyableAssets(assetIds: [String]) throws -> Int {
        try updateColumn(column: AssetRecord.Columns.isBuyable, enabledAssetIds: assetIds)
    }

    @discardableResult
    public func updateSellableAssets(assetIds: [String]) throws -> Int {
        try updateColumn(column: AssetRecord.Columns.isSellable, enabledAssetIds: assetIds)
    }

    private func updateColumn(column: Column, enabledAssetIds: [String]) throws -> Int {
        try db.write { db in
            let enabled = try AssetRecord
                .filter(enabledAssetIds.contains(AssetRecord.Columns.id) && column == false)
                .updateAll(db, column.set(to: true))
            let disabled = try AssetRecord
                .filter(!enabledAssetIds.contains(AssetRecord.Columns.id) && column == true)
                .updateAll(db, column.set(to: false))
            return enabled + disabled
        }
    }

    private func setColumn(for assetIds: [String], column: Column, value: Bool) throws -> Int {
        try db.write { db in
            try AssetRecord
                .filter(assetIds.contains(AssetRecord.Columns.id) && column != value)
                .updateAll(db, column.set(to: value))
        }
    }

    @discardableResult
    public func clearTokens() throws -> Int {
        try db.write { db in
            try AssetRecord
                .filter(AssetRecord.Columns.type != AssetType.native.rawValue)
                .deleteAll(db)
        }
    }

    public func updateLinks(assetId: AssetId, _ links: [AssetLink]) throws {
        try db.write { db in
            for link in links {
                try link.record(assetId: assetId).upsert(db)
            }
        }
    }

    public func updateAssociations(assetId: AssetId, associations: [AssetAssociation]) throws {
        try db.write { db in
            guard var record = try AssetRecord
                .filter(AssetRecord.Columns.id == assetId.identifier)
                .fetchOne(db)
            else { return }
            record.associations = associations
            try record.update(db, columns: [AssetRecord.Columns.associations])
        }
    }

}
