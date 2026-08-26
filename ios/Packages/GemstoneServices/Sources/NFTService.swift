// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemNftServiceProtocol
import GemstonePrimitives
import Primitives

public struct NFTService: Sendable {
    private let service: any GemNftServiceProtocol

    public init(service: any GemNftServiceProtocol) {
        self.service = service
    }

    @discardableResult
    public func updateAssets(wallet: Wallet) async throws -> Int {
        Int(try await service.sync(walletId: wallet.id.id))
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
        try NFTAssetData(await service.getOrFetchAsset(assetId: assetId.identifier))
    }
}
