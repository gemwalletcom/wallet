// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemAssetDiscoveryStore
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
}
