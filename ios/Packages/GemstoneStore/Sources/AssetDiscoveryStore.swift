// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemAssetDiscoveryStore
import enum Gemstone.GemDiscoveryStep
import Preferences
import Primitives

public final class GemstoneAssetDiscoveryStore: GemAssetDiscoveryStore, @unchecked Sendable {
    public init() {}

    public func getAssetsTimestamp(walletId: String) async throws -> UInt64 {
        try UInt64(WalletPreferences(walletId: WalletId.from(id: walletId)).assetsTimestamp)
    }

    public func setAssetsTimestamp(walletId: String, timestamp: UInt64) async throws {
        try WalletPreferences(walletId: WalletId.from(id: walletId)).assetsTimestamp = Int(timestamp)
    }

    public func isCompleted(walletId: String, step: GemDiscoveryStep) async throws -> Bool {
        let preferences = try WalletPreferences(walletId: WalletId.from(id: walletId))
        return switch step {
        case .assets: preferences.completeInitialLoadAssets
        case .transactions: preferences.completeInitialLoadTransactions
        case .nfts: preferences.completeInitialLoadNFTs
        }
    }

    public func setCompleted(walletId: String, step: GemDiscoveryStep) async throws {
        let preferences = try WalletPreferences(walletId: WalletId.from(id: walletId))
        switch step {
        case .assets: preferences.completeInitialLoadAssets = true
        case .transactions: preferences.completeInitialLoadTransactions = true
        case .nfts: preferences.completeInitialLoadNFTs = true
        }
    }
}
