// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Localization
import Primitives

struct TransactionExplorerViewModel {
    private let transactionLink: BlockExplorerLink

    init(transactionLink: BlockExplorerLink) {
        self.transactionLink = transactionLink
    }

    var url: URL {
        transactionLink.url
    }
}

// MARK: - ItemModelProvidable

extension TransactionExplorerViewModel: ItemModelProvidable {
    var itemModel: TransactionItemModel {
        .explorer(
            url: transactionLink.url,
            text: Localized.Transaction.viewOn(transactionLink.name),
        )
    }
}
