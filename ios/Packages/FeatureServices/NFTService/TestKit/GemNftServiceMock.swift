// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import GemstonePrimitives
import Primitives

public final class GemNftServiceMock: GemNftServiceProtocol, @unchecked Sendable {
    private let lock = NSLock()
    private var assets: [Primitives.NFTData]
    private let store: (any GemNftStore)?

    public init(assets: [Primitives.NFTData] = [], store: (any GemNftStore)? = nil) {
        self.assets = assets
        self.store = store
    }

    public func setAssets(_ assets: [Primitives.NFTData]) {
        lock.withLock { self.assets = assets }
    }

    public func sync(walletId: String) async throws -> UInt32 {
        let assets = lock.withLock { self.assets }
        try await store?.save(walletId: walletId, data: assets.map { try $0.json() })
        return UInt32(assets.count)
    }

    public func getOrFetchAsset(assetId: Gemstone.NftAssetId) async throws -> Gemstone.NftAssetData {
        guard let data = try await store?.getAssetData(assetId: assetId) else {
            throw AnyError("not stubbed")
        }
        return data
    }

    public func refreshAsset(walletId _: String, assetId _: Gemstone.NftAssetId) async throws {}

    public func report(report _: Gemstone.ReportNft) async throws {}
}
