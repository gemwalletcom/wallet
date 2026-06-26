// Copyright (c). Gem Wallet. All rights reserved.

import BalanceService
import Foundation
import Primitives

public struct BalanceUpdaterMock: BalanceUpdater {
    private let updateBalance: @Sendable (Wallet, [AssetId]) async -> Void

    public init(updateBalance: @escaping @Sendable (Wallet, [AssetId]) async -> Void = { _, _ in }) {
        self.updateBalance = updateBalance
    }

    public func updateBalance(for wallet: Wallet, assetIds: [AssetId]) async {
        await updateBalance(wallet, assetIds)
    }
}

public extension BalanceUpdater where Self == BalanceUpdaterMock {
    static func mock() -> BalanceUpdaterMock {
        BalanceUpdaterMock()
    }
}
