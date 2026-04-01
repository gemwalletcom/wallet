// Copyright (c). Gem Wallet. All rights reserved.

import BalanceService
import Foundation
import Primitives

public struct BalanceUpdaterMock: BalanceUpdater {
    public init() {}
    public func updateBalance(for _: Wallet, assetIds _: [AssetId]) async {}
}

public extension BalanceUpdater where Self == BalanceUpdaterMock {
    static func mock() -> BalanceUpdaterMock {
        BalanceUpdaterMock()
    }
}
