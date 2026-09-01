// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Store

public protocol RecentAssetsServiceable: Sendable {
    func add(_ data: RecentActivityData, walletId: String) throws
    func clear(walletId: String, types: [RecentActivityType]) throws
}

public struct RecentAssetsService: RecentAssetsServiceable {
    private let store: RecentActivityStore

    public init(store: RecentActivityStore) {
        self.store = store
    }

    public func add(_ data: RecentActivityData, walletId: String) throws {
        try store.add(data, walletId: walletId)
    }

    public func clear(walletId: String, types: [RecentActivityType]) throws {
        try store.clear(walletId: walletId, types: types)
    }
}
