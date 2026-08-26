// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemWalletConfigurationStore
import Preferences
import Primitives

public final class GemstoneWalletConfigurationStore: GemWalletConfigurationStore, @unchecked Sendable {
    public init() {}

    public func isCompleted(walletId: String) async throws -> Bool {
        try WalletPreferences(walletId: WalletId.from(id: walletId)).completeInitialWalletConfiguration
    }

    public func setCompleted(walletId: String) async throws {
        try WalletPreferences(walletId: WalletId.from(id: walletId)).completeInitialWalletConfiguration = true
    }
}
