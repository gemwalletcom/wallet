// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.pendingActivityFilters
import GemstonePrimitives
import Primitives
import Store
import Transactions

@Observable
@MainActor
final class MainTabViewModel {
    let transactionsQuery: ObservableQuery<TransactionsCountQuery>

    var transactions: Int {
        transactionsQuery.value
    }

    var isPresentingToastMessage: ToastMessage?

    init(wallet: Wallet) {
        transactionsQuery = ObservableQuery(
            TransactionsCountQuery(walletId: wallet.id, type: .all, filter: pendingActivityFilters().toPrimitives()),
            initialValue: 0,
        )
    }
}
