// Copyright (c). Gem Wallet. All rights reserved.

import BalanceService
import BalanceServiceTestKit
import Blockchain
import ChainService
import ChainServiceTestKit
import EarnService
import Foundation
import Gemstone
import NativeProviderService
import NFTService
import NFTServiceTestKit
import Primitives
import StakeService
import StakeServiceTestKit
import Store
import StoreTestKit
import SwapServiceTestKit
import TransactionStateService

public extension TransactionStateService {
    static func mock(
        transactionStore: TransactionStore = .mock(),
        swapper: any GemSwapperProtocol = GemSwapperMock(),
        stakeService: StakeService = .mock(),
        earnService: EarnService = .mock(),
        nftService: NFTService = .mock(),
        chainServiceFactory: any ChainServiceFactorable = ChainServiceFactoryMock(),
    ) -> TransactionStateService {
        TransactionStateService(
            transactionStore: transactionStore,
            swapper: swapper,
            stakeService: stakeService,
            earnService: earnService,
            nftService: nftService,
            chainServiceFactory: chainServiceFactory,
            balanceUpdater: .mock(),
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
