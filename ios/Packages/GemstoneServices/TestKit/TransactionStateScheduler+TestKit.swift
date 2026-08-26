// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemNftService
import protocol Gemstone.GemNftServiceProtocol
import GemstoneServices
import Foundation
import protocol Gemstone.GemStakeServiceProtocol
import GemstonePrimitivesTestKit
import Store
import StoreTestKit

public extension TransactionStateScheduler {
    static func mock(
        transactionStore: TransactionStore = .mock(),
        gatewayService: GatewayService = .mock(),
        stakeService: any GemStakeServiceProtocol = GemStakeServiceMock(),
        nftService: any GemNftServiceProtocol = GemNftService.mock(),
    ) -> TransactionStateScheduler {
        let postProcessingService = TransactionPostProcessingService(
            balanceService: GemBalanceServiceMock(),
            stakeService: stakeService,
            nftService: nftService,
        )
        let service = TransactionStateService(
            service: gatewayService.transactionStateService(store: GemstoneTransactionStateStore(store: transactionStore)),
            postProcessingService: postProcessingService,
        )
        return TransactionStateScheduler(service: service)
    }
}
