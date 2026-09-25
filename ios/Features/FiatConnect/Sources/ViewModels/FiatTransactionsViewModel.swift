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

    public let query: ObservableQuery<MappedRequest<FiatTransactionsRequest, [ListSection<FiatTransactionViewModel>]>>

    private var loadState: GemLoadState = .loading
    var sections: [ListSection<FiatTransactionViewModel>] {
        query.value
    }

    init(walletId: WalletId, service: any GemFiatQuoteServiceProtocol) {
        self.walletId = walletId
        self.service = service
        query = ObservableQuery(
            MappedRequest(FiatTransactionsRequest(walletId: walletId)) {
                DateSectionBuilder(items: FiatTransactionViewModel.models($0), dateKeyPath: \.createdAt).build()
            },
            initialValue: [],
        )
    }

    var title: String {
        Localized.Activity.title
    }

    var loadError: Error? {
        Gemstone.loadError(state: loadState, hasRows: !sections.isEmpty)
    }

    var emptyContentModel: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: EmptyContentType(.activity))
    }

    func load() async {
        loadState = await service.refreshTransactions(hasTransactions: !sections.isEmpty)
    }
}
