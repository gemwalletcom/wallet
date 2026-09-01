// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Store

public protocol RecentAssetsServiceable: Sendable {
    func add(_ data: RecentActivityData, walletId: WalletId) throws
    func clear(walletId: WalletId, types: [RecentActivityType]) throws
}

public struct RecentAssetsService: RecentAssetsServiceable {
    private let store: RecentActivityStore

    public init(store: RecentActivityStore) {
        self.store = store
    }

    public func add(_ data: RecentActivityData, walletId: WalletId) throws {
        try store.add(data, walletId: walletId)
    }

    public func clear(walletId: WalletId, types: [RecentActivityType]) throws {
        try store.clear(walletId: walletId, types: types)
    }
}
