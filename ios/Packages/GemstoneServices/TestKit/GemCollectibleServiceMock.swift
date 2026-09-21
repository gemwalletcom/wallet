// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone

public final class GemCollectibleServiceMock: GemCollectibleServiceProtocol, @unchecked Sendable {
    public var reportError: Error?
    public private(set) var reports: [(assetId: NftAssetId, reason: ReportReason)] = []

    public init() {}

    public func details(walletType: WalletType, assetData: NftAssetData, isOwned: Bool, canSaveImage: Bool) -> GemCollectibleDetails {
        GemCollectibleService.mock().details(walletType: walletType, assetData: assetData, isOwned: isOwned, canSaveImage: canSaveImage)
    }

    public func refreshAsset(assetId _: NftAssetId) async throws {}

    public func report(assetId: NftAssetId, reason: ReportReason) async throws {
        reports.append((assetId, reason))
        if let reportError {
            throw reportError
        }
    }

    public func setWalletAvatar(url _: String) async throws {}
}
