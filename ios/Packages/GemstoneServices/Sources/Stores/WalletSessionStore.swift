// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Observation
import Primitives
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
        get {
            access(keyPath: \.currentWalletId)
            return store.get(key: Self.key)
        }
        set {
            withMutation(keyPath: \.currentWalletId) {
                do {
                    switch newValue {
                    case let .some(walletId): try store.set(key: Self.key, value: walletId)
                    case .none: try store.remove(key: Self.key)
                    }
                } catch {
                    debugLog("wallet session store write error: \(error)")
                }
            }
        }
    }

    public func getCurrentWalletId() throws -> Gemstone.WalletId? {
        currentWalletId
    }

    public func setCurrentWalletId(walletId: Gemstone.WalletId?) throws {
        if Thread.isMainThread {
            currentWalletId = walletId
        } else {
            DispatchQueue.main.sync {
                currentWalletId = walletId
            }
        }
    }
}
