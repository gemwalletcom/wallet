// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Primitives
import Store
import Transactions

@Observable
@MainActor
final class MainTabViewModel {
    let transactionsQuery: ObservableQuery<TransactionsCountRequest>

    var transactions: Int {
        transactionsQuery.value
    }

    var isPresentingToastMessage: ToastMessage?

    init(wallet: Wallet) {
        transactionsQuery = ObservableQuery(
            TransactionsCountRequest(walletId: wallet.id, type: .pending, filters: TransactionsRequestFilter.activityDefaults),
            initialValue: 0,
        )
    }
}
