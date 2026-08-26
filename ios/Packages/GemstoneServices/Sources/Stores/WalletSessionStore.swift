// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemWalletSessionStore
import typealias Gemstone.WalletId
import Preferences

public final class GemstoneWalletSessionStore: GemWalletSessionStore, @unchecked Sendable {
    private let preferences: ObservablePreferences

    public init(preferences: ObservablePreferences) {
        self.preferences = preferences
    }

    public func getCurrentWalletId() throws -> Gemstone.WalletId? {
        preferences.currentWalletId
    }

    public func setCurrentWalletId(walletId: Gemstone.WalletId?) throws {
        preferences.currentWalletId = walletId
    }
}
