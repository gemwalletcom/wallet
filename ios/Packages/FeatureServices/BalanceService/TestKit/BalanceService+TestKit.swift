// Copyright (c). Gem Wallet. All rights reserved.

import BalanceService
import Foundation
import protocol Gemstone.GemBalanceServiceProtocol
import GemstonePrimitivesTestKit
import Store
import StoreTestKit

public extension BalanceService {
    static func mock(
        balanceStore: BalanceStore = .mock(),
        service: any GemBalanceServiceProtocol = GemBalanceServiceMock(),
    ) -> BalanceService {
        BalanceService(
            balanceStore: balanceStore,
            service: service,
        )
    }
}
