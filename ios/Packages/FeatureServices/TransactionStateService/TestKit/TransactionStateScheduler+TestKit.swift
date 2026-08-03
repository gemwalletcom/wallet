// Copyright (c). Gem Wallet. All rights reserved.

import BalanceService
import BalanceServiceTestKit
import Blockchain
import EarnService
import Foundation
import NativeProviderService
import NFTService
import NFTServiceTestKit
import PaymentService
import PaymentServiceTestKit
import Primitives
import StakeService
import StakeServiceTestKit
import Store
import StoreTestKit
import TransactionStateService

public extension TransactionStateScheduler {
    static func mock(
        transactionStore: TransactionStore = .mock(),
        gatewayService: GatewayService = GatewayService(provider: NativeProvider(url: Constants.apiURL, requestInterceptor: EmptyRequestInterceptor())),
        stakeService: StakeService = .mock(),
        earnService: EarnService = .mock(),
        nftService: NFTService = .mock(),
        paymentStatusService: any PaymentStatusServiceable = PaymentStatusServiceableMock(),
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
            gatewayService: gatewayService,
            postProcessingService: postProcessingService,
            paymentStatusService: paymentStatusService,
        )
        return TransactionStateScheduler(
            transactionStore: transactionStore,
            service: service,
        )
    }
}

public extension EarnService {
    static func mock(
        store: StakeStore = .mock(),
    ) -> EarnService {
        let provider = NativeProvider(url: Constants.apiURL, requestInterceptor: EmptyRequestInterceptor())
        return EarnService(store: store, gatewayService: GatewayService(provider: provider))
    }
}
