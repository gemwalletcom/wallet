// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import protocol Gemstone.GemFiatQuoteServiceProtocol
import enum Gemstone.GemLoadState
import func Gemstone.loadError
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import Store

@Observable
@MainActor
public final class FiatTransactionsViewModel {
    private let service: any GemFiatQuoteServiceProtocol
    let walletId: WalletId

    public let query: ObservableQuery<FiatTransactionsRequest>

    private var loadState: GemLoadState = .loading
    var transactions: [FiatTransactionAssetData] {
        query.value
    }

    var sections: [ListSection<FiatTransactionAssetData>] {
        DateSectionBuilder(items: transactions, dateKeyPath: \.createdAt).build()
    }

    init(walletId: WalletId, service: any GemFiatQuoteServiceProtocol) {
        self.walletId = walletId
        self.service = service
        query = ObservableQuery(FiatTransactionsRequest(walletId: walletId), initialValue: [])
    }

    var title: String {
        Localized.Activity.title
    }

    var loadError: Error? {
        Gemstone.loadError(state: loadState, hasRows: !transactions.isEmpty)
    }

    var emptyContentModel: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: .activity(isViewOnly: false))
    }

    func load() async {
        loadState = await service.refreshTransactions(hasTransactions: transactions.isNotEmpty)
    }
}
