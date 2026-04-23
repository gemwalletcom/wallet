// Copyright (c). Gem Wallet. All rights reserved.

import Blockchain
import EarnService
import protocol Gemstone.GemTransactionStateServiceProtocol
import struct Gemstone.TransactionStateInput
import NativeProviderService
import Primitives
import StakeService
import StakeServiceTestKit
import Store
import StoreTestKit
import TransactionStateService

public extension TransactionStateService {
    static func mock(
        service: GemTransactionStateServiceProtocol = GemTransactionStateServiceMock(),
        transactionStore: TransactionStore = .mock(),
    ) -> TransactionStateService {
        TransactionStateService(service: service, transactionStore: transactionStore)
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

public final class GemTransactionStateServiceMock: GemTransactionStateServiceProtocol, @unchecked Sendable {
    public init() {}

    public func monitor(inputs _: [TransactionStateInput]) async {}

    public func stopMonitoring() async {}
}
