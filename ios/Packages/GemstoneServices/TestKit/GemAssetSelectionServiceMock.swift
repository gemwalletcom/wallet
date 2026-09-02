// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.AssetBasic
import typealias Gemstone.AssetId
import typealias Gemstone.Currency
import protocol Gemstone.GemAssetSelectionServiceProtocol
import enum Gemstone.RecentActivityType
import typealias Gemstone.Wallet
import typealias Gemstone.WalletId
import Primitives

public final class GemAssetSelectionServiceMock: GemAssetSelectionServiceProtocol, @unchecked Sendable {
    private let assets: [AssetBasic]
    private let error: Error?
    private let onSetAssetsEnabled: (@Sendable (WalletId, [AssetId], Bool) -> Void)?

    public init(
        assets: [AssetBasic] = [],
        error: Error? = nil,
        onSetAssetsEnabled: (@Sendable (WalletId, [AssetId], Bool) -> Void)? = nil,
    ) {
        self.assets = assets
        self.error = error
        self.onSetAssetsEnabled = onSetAssetsEnabled
    }

    public func currency() -> Currency {
        Primitives.Currency.usd.json()
    }

    public func searchAssets(wallet: Wallet, query: String) async throws -> [AssetBasic] {
        if let error { throw error }
        return assets
    }

    public func setAssetsEnabled(walletId: WalletId, assetIds: [AssetId], enabled: Bool) async throws {
        onSetAssetsEnabled?(walletId, assetIds, enabled)
    }

    public func addRecentAsset(activityType: RecentActivityType, assetId: AssetId, walletId: WalletId) async throws {}

    public func setPriceAlert(assetId: AssetId, enabled: Bool) async throws {}
}
