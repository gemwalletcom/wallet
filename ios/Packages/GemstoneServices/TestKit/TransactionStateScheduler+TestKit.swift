// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import Blockchain
import BlockchainTestKit
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
        nftService: NFTService = .mock(),
    ) -> TransactionStateScheduler {
        let postProcessingService = TransactionPostProcessingService(
            transactionStore: transactionStore,
            balanceUpdater: .mock(),
            stakeService: stakeService,
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
