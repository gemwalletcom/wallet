// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemPreferencesStore
import protocol Gemstone.GemWalletSessionStore
import Observation
import Primitives

@Observable
public final class GemstoneWalletSessionStore: GemWalletSessionStore, @unchecked Sendable {
    private static let key = "current_wallet_id"

    private let store: any GemPreferencesStore

    public init(store: any GemPreferencesStore) {
        self.store = store
    }

    @ObservationIgnored
    public var currentWalletId: WalletId? {
        access(keyPath: \.currentWalletId)
        return store.get(key: Self.key).flatMap { try? WalletId.from(id: $0) }
    }

    public func getCurrentWalletId() throws -> WalletId? {
        currentWalletId
    }

    public func setCurrentWalletId(walletId: WalletId?) throws {
        switch walletId {
        case let .some(walletId): try store.set(key: Self.key, value: walletId.id)
        case .none: try store.remove(key: Self.key)
        }
        withMutation(keyPath: \.currentWalletId) {}
    }
}
