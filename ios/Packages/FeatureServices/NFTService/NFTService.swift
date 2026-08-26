// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemNftServiceProtocol
import GemstonePrimitives
import Primitives
import Store

public struct NFTService: Sendable {
    private let service: any GemNftServiceProtocol
    private let nftStore: NFTStore

    public init(
        service: any GemNftServiceProtocol,
        nftStore: NFTStore,
    ) {
        self.service = service
        self.nftStore = nftStore
    }

    @discardableResult
    public func updateAssets(wallet: Wallet) async throws -> Int {
        let nfts = try await service.getAssets(walletId: wallet.id.id).map { try NFTData($0) }
        try nftStore.save(nfts, for: wallet.id)
        return nfts.count
    }

    public func report(collectionId: NFTCollectionId, assetId: NFTAssetId?, reason: String?) async throws {
        let report = ReportNft(
            collectionId: collectionId.identifier,
            assetId: assetId?.identifier,
            reason: reason,
        )
        try await service.report(report: report.json())
    }

    public func refreshAsset(wallet: Wallet, assetId: NFTAssetId) async throws {
        try await service.refreshAsset(walletId: wallet.id.id, assetId: assetId.identifier)
    }

    public func getOrFetchAssetData(assetId: NFTAssetId) async throws -> NFTAssetData {
        if let asset = try nftStore.getAsset(assetId: assetId),
           let collection = try nftStore.getCollection(collectionId: asset.collectionId)
        {
            return NFTAssetData(collection: collection, asset: asset)
        }
        let assetData = try await NFTAssetData(service.getAsset(assetId: assetId.identifier))
        try nftStore.add(asset: assetData.asset, collection: assetData.collection)
        return assetData
    }
}
