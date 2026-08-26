// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import Foundation
import Primitives

public struct AssetsEnablerMock: AssetsEnabler {
    private let onEnableAssets: (@Sendable (Wallet, [AssetId], Bool) async throws -> Void)?
    private let onPinAsset: (@Sendable (Wallet, AssetId, Bool) async throws -> Void)?

    public init(
        onEnableAssets: (@Sendable (Wallet, [AssetId], Bool) async throws -> Void)? = nil,
        onPinAsset: (@Sendable (Wallet, AssetId, Bool) async throws -> Void)? = nil,
    ) {
        self.onEnableAssets = onEnableAssets
        self.onPinAsset = onPinAsset
    }

    public func enableAssets(wallet: Wallet, assetIds: [AssetId], enabled: Bool) async throws {
        try await onEnableAssets?(wallet, assetIds, enabled)
    }

    public func pinAsset(wallet: Wallet, assetId: AssetId, pinned: Bool) async throws {
        try await onPinAsset?(wallet, assetId, pinned)

        guard pinned else { return }
        try await enableAssets(wallet: wallet, assetIds: [assetId], enabled: true)
    }
}

public extension AssetsEnabler where Self == AssetsEnablerMock {
    static func mock(
        onEnableAssets: (@Sendable (Wallet, [AssetId], Bool) async throws -> Void)? = nil,
        onPinAsset: (@Sendable (Wallet, AssetId, Bool) async throws -> Void)? = nil,
    ) -> AssetsEnablerMock {
        AssetsEnablerMock(onEnableAssets: onEnableAssets, onPinAsset: onPinAsset)
    }
}
