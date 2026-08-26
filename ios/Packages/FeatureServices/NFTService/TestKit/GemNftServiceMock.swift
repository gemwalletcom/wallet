// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import GemstonePrimitives
import Primitives

public final class GemNftServiceMock: GemNftServiceProtocol, @unchecked Sendable {
    private let lock = NSLock()
    private var assets: [Primitives.NFTData]

    public init(assets: [Primitives.NFTData] = []) {
        self.assets = assets
    }

    public func setAssets(_ assets: [Primitives.NFTData]) {
        lock.withLock { self.assets = assets }
    }

    public func getAssets(walletId _: String) async throws -> [Gemstone.NftData] {
        try lock.withLock { assets }.map { try $0.json() }
    }

    public func getAsset(assetId _: Gemstone.NftAssetId) async throws -> Gemstone.NftAssetData {
        throw AnyError("not stubbed")
    }

    public func refreshAsset(walletId _: String, assetId _: Gemstone.NftAssetId) async throws {}

    public func report(report _: Gemstone.ReportNft) async throws {}
}
