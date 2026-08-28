// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Observation
import protocol Gemstone.GemPreferencesStore
import protocol Gemstone.GemWalletSessionStore
import typealias Gemstone.WalletId

@Observable
public final class GemstoneWalletSessionStore: GemWalletSessionStore, @unchecked Sendable {
    private static let key = "current_wallet_id"

    private let store: any GemPreferencesStore

    public init(store: any GemPreferencesStore) {
        self.store = store
    }

    @ObservationIgnored
    public var currentWalletId: Gemstone.WalletId? {
        access(keyPath: \.currentWalletId)
        return store.get(key: Self.key)
    }

    public func getCurrentWalletId() throws -> Gemstone.WalletId? {
        currentWalletId
    }

    public func setCurrentWalletId(walletId: Gemstone.WalletId?) throws {
        switch walletId {
        case let .some(walletId): try store.set(key: Self.key, value: walletId)
        case .none: try store.remove(key: Self.key)
        }
        withMutation(keyPath: \.currentWalletId) {}
    }
}
