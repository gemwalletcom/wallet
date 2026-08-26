// Copyright (c). Gem Wallet. All rights reserved.

import BalanceService
import BalanceServiceTestKit
import Blockchain
import BlockchainTestKit
import EarnService
import EarnServiceTestKit
import Foundation
import NFTService
import NFTServiceTestKit
import StakeService
import StakeServiceTestKit
import Store
import StoreTestKit
import TransactionStateService

public extension TransactionStateScheduler {
    static func mock(
        transactionStore: TransactionStore = .mock(),
        gatewayService: GatewayService = .mock(),
        stakeService: StakeService = .mock(),
        earnService: any EarnPositionsUpdatable = .mock(),
        nftService: NFTService = .mock(),
    ) -> TransactionStateScheduler {
        let postProcessingService = TransactionPostProcessingService(
            transactionStore: transactionStore,
            balanceUpdater: .mock(),
            stakeService: stakeService,
            earnService: earnService,
            nftService: nftService,
        )
        let service = TransactionStateService(
            transactionStore: transactionStore,
            service: gatewayService.transactionStateService(store: GemstoneTransactionStateStore(store: transactionStore)),
            postProcessingService: postProcessingService,
        )
        return TransactionStateScheduler(
            transactionStore: transactionStore,
            service: service,
        )
    }
}
