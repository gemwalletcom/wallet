// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemAPI
import Primitives
import Store

public struct NFTService: Sendable {
    private let apiService: any GemAPINFTService
    private let nftStore: NFTStore

    public init(
        apiService: any GemAPINFTService,
        nftStore: NFTStore,
    ) {
        self.apiService = apiService
        self.nftStore = nftStore
    }

    @discardableResult
    public func updateAssets(wallet: Wallet) async throws -> Int {
        let nfts = try await apiService.getDeviceNFTAssets(walletId: wallet.id)
        try nftStore.save(nfts, for: wallet.id)
        return nfts.count
    }

    public func report(collectionId: NFTCollectionId, assetId: NFTAssetId?, reason: String?) async throws {
        let report = ReportNft(
            collectionId: collectionId.identifier,
            assetId: assetId?.identifier,
            reason: reason,
        )
        try await apiService.reportNft(report: report)
    }

    public func refreshAsset(wallet: Wallet, assetId: NFTAssetId) async throws {
        try await apiService.refreshNftAsset(walletId: wallet.id, assetId: assetId)
    }

    public func getOrFetchAssetData(assetId: NFTAssetId) async throws -> NFTAssetData {
        if let asset = try nftStore.getAsset(assetId: assetId),
           let collection = try nftStore.getCollection(collectionId: asset.collectionId)
        {
            return NFTAssetData(collection: collection, asset: asset)
        }
        let assetData = try await apiService.getDeviceNFTAsset(assetId: assetId)
        try nftStore.add(asset: assetData.asset, collection: assetData.collection)
        return assetData
    }
}
