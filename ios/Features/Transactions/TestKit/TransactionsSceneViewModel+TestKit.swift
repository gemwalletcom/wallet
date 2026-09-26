// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemTransactionsServiceProtocol
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
import Store
import Transactions

public extension TransactionsSceneViewModel {
    @MainActor
    static func mock(
        service: any GemTransactionsServiceProtocol = GemTransactionsServiceMock(),
    ) -> TransactionsSceneViewModel {
        TransactionsSceneViewModel(service: service, wallet: .mock(), type: .all)
    }
}
