// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemRecentActivity
import protocol Gemstone.GemRecentActivityStore
import enum Gemstone.RecentActivityType
import typealias Gemstone.WalletId
import GemstonePrimitives
import Primitives
import Store

public final class GemstoneRecentActivityStore: GemRecentActivityStore, @unchecked Sendable {
    private let store: RecentActivityStore

    public init(store: RecentActivityStore) {
        self.store = store
    }

    public func add(activity: GemRecentActivity, walletId: Gemstone.WalletId) async throws {
        try store.add(RecentActivityData(activity), walletId: Primitives.WalletId.from(id: walletId))
    }

    public func clear(walletId: Gemstone.WalletId, types: [Gemstone.RecentActivityType]) async throws {
        try store.clear(walletId: Primitives.WalletId.from(id: walletId), types: types.map { $0.toPrimitives() })
    }
}
