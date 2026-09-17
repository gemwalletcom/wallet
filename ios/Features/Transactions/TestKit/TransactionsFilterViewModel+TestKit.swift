// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Store
import Transactions

public extension TransactionsFilterViewModel {
    @MainActor
    static func mock() -> TransactionsFilterViewModel {
        TransactionsFilterViewModel(wallet: .mock(), chains: [.bitcoin, .ethereum], type: .all)
    }
}
