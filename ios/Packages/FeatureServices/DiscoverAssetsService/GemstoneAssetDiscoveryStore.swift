// Copyright (c). Gem Wallet. All rights reserved.

import BalanceService
import Foundation
import typealias Gemstone.AssetId
import protocol Gemstone.GemAssetDiscoveryStore
import Preferences
import Primitives
import Store

public final class GemstoneAssetDiscoveryStore: GemAssetDiscoveryStore, @unchecked Sendable {
    private let walletStore: WalletStore
    private let assetsEnabler: any AssetsEnabler

    public init(walletStore: WalletStore, assetsEnabler: any AssetsEnabler) {
        self.walletStore = walletStore
        self.assetsEnabler = assetsEnabler
    }

    public func getAssetsTimestamp(walletId: String) async throws -> UInt64 {
        try UInt64(WalletPreferences(walletId: WalletId.from(id: walletId)).assetsTimestamp)
    }

    public func setAssetsTimestamp(walletId: String, timestamp: UInt64) async throws {
        try WalletPreferences(walletId: WalletId.from(id: walletId)).assetsTimestamp = Int(timestamp)
    }

    public func enableAssets(walletId: String, assetIds: [Gemstone.AssetId]) async throws {
        guard let wallet = try walletStore.getWallet(id: WalletId.from(id: walletId)) else { return }
        try await assetsEnabler.enableAssets(wallet: wallet, assetIds: assetIds.map { try Primitives.AssetId(id: $0) }, enabled: true)
    }
}
