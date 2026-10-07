// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemCollectibleDetails
import class Gemstone.GemCollectibleService
import protocol Gemstone.GemCollectibleServiceProtocol
import struct Gemstone.NftAssetData
import typealias Gemstone.NftAssetId
import enum Gemstone.ReportReason
import enum Gemstone.WalletType
import GemstonePrimitives
import Primitives
import PrimitivesTestKit

public final class GemCollectibleServiceMock: GemCollectibleServiceProtocol, @unchecked Sendable {
    public var ensureAssetResult: Result<NftAssetData, Error>
    public var reportError: Error?
    public private(set) var reports: [(assetId: NftAssetId, reason: ReportReason)] = []

    public init(ensureAssetResult: Result<NftAssetData, Error> = .success(NFTAssetData.mock().toGem())) {
        self.ensureAssetResult = ensureAssetResult
    }

    public func details(walletType: WalletType, assetData: NftAssetData, isOwned: Bool, canSaveImage: Bool) -> GemCollectibleDetails {
        GemCollectibleService.mock().details(walletType: walletType, assetData: assetData, isOwned: isOwned, canSaveImage: canSaveImage)
    }

    public func ensureAsset(assetId _: NftAssetId) async throws -> NftAssetData {
        try ensureAssetResult.get()
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
