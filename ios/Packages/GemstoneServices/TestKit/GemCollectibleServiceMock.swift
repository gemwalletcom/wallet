// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone

public final class GemCollectibleServiceMock: GemCollectibleServiceProtocol, @unchecked Sendable {
    public var reportError: Error?
    public private(set) var reports: [ReportNft] = []

    public init() {}

    public func details(walletType: WalletType, assetData: NftAssetData, isOwned: Bool) -> GemCollectibleDetails {
        GemCollectibleService.mock().details(walletType: walletType, assetData: assetData, isOwned: isOwned)
    }

    public func refreshAsset(assetId _: NftAssetId) async throws {}

    public func report(report: ReportNft) async throws {
        reports.append(report)
        if let reportError { throw reportError }
    }

    public func setWalletAvatar(url _: String) async throws {}
}
