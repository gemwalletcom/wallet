// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import protocol Gemstone.GemExplorerServiceProtocol
import Components
import Foundation
import Localization
import Primitives
import PrimitivesComponents

struct TransactionExplorerViewModel {
    private let transactionViewModel: TransactionViewModel
    private let explorerService: any GemExplorerServiceProtocol

    init(
        transactionViewModel: TransactionViewModel,
        explorerService: any GemExplorerServiceProtocol,
    ) {
        self.transactionViewModel = transactionViewModel
        self.explorerService = explorerService
    }

    private var transactionLink: BlockExplorerLink {
        BlockExplorerLink(explorerService.getTransactionLink(
            chain: transactionViewModel.transaction.transaction.assetId.chain.rawValue,
            hash: transactionViewModel.transaction.transaction.id.hash,
            provider: transactionViewModel.transaction.transaction.swapProvider,
            recipient: transactionViewModel.transaction.transaction.to,
            memo: transactionViewModel.transaction.transaction.memo,
        ))
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
