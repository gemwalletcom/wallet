// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Store
import Transactions

public extension TransactionsFilterSceneViewModel {
    @MainActor
    static func mock() -> TransactionsFilterSceneViewModel {
        TransactionsFilterSceneViewModel(wallet: .mock(), chains: [.bitcoin, .ethereum], type: .all)
    }
}
