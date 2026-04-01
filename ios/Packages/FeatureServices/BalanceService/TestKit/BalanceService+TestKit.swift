// Copyright (c). Gem Wallet. All rights reserved.

import AssetsService
import AssetsServiceTestKit
import BalanceService
import ChainService
import ChainServiceTestKit
import Foundation
import Store
import StoreTestKit

public extension BalanceService {
    static func mock(
        balanceStore: BalanceStore = .mock(),
        assetsService: AssetsService = .mock(),
        chainServiceFactory: any ChainServiceFactorable = ChainServiceFactoryMock(),
    ) -> BalanceService {
        BalanceService(
            balanceStore: balanceStore,
            assetsService: assetsService,
            chainServiceFactory: chainServiceFactory,
        )
    }
}
