// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemAPI
import Primitives

public final class GemAPINFTServiceMock: GemAPINFTService, @unchecked Sendable {
    private var nftAssets: [NFTData]

    public init(nftAssets: [NFTData] = []) {
        self.nftAssets = nftAssets
    }

    public func getDeviceNFTAssets(walletId _: String) async throws -> [NFTData] { nftAssets }

    public func reportNft(report _: ReportNft) async throws {}

    public func setNFTAssets(_ nftAssets: [NFTData]) {
        self.nftAssets = nftAssets
    }
}
