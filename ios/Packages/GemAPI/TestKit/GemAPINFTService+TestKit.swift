// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemAPI
import Primitives

public final class GemAPINFTServiceMock: GemAPINFTService, @unchecked Sendable {
    private var nftAssets: [NFTData]

    public init(nftAssets: [NFTData] = []) {
        self.nftAssets = nftAssets
    }

    public func getDeviceNFTAssets(walletId _: WalletId) async throws -> [NFTData] {
        nftAssets
    }

    public func getDeviceNFTAsset(assetId: String) async throws -> NFTAssetData {
        guard let assetData = nftAssets.assetData(for: assetId) else {
            throw AnyError("NFT asset not found")
        }
        return assetData
    }

    public func refreshNftAsset(walletId _: WalletId, assetId _: String) async throws {}

    public func reportNft(report _: ReportNft) async throws {}

    public func setNFTAssets(_ nftAssets: [NFTData]) {
        self.nftAssets = nftAssets
    }
}

private extension [NFTData] {
    func assetData(for assetId: String) -> NFTAssetData? {
        for data in self {
            if let asset = data.assets.first(where: { $0.id == assetId }) {
                return NFTAssetData(collection: data.collection, asset: asset)
            }
        }
        return nil
    }
}
