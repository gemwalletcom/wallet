// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Primitives
import PrimitivesComponents

struct TransactionMemoViewModel: Sendable {
    private let transaction: Transaction

    init(transaction: Transaction) {
        self.transaction = transaction
    }
}

// MARK: - ItemModelProvidable

extension TransactionMemoViewModel: ItemModelProvidable {
    var itemModel: TransactionItemModel {
        guard transaction.assetId.chain.isMemoSupported, transaction.memo?.isEmpty == false else { return .empty }

        return .listItem(
            MemoViewModel(memo: transaction.memo).listItemModel,
        )
    }
}
