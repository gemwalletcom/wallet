// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.fiatTransactionRows
import protocol Gemstone.GemFiatQuoteServiceProtocol
import struct Gemstone.GemFiatTransactionRow
import enum Gemstone.GemLoadState
import func Gemstone.loadError
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import Store

@Observable
@MainActor
public final class FiatTransactionsSceneViewModel {
    private let service: any GemFiatQuoteServiceProtocol
    let walletId: WalletId

    public let query: ObservableQuery<MappedQuery<FiatTransactionsQuery, [ListSection<GemFiatTransactionRow>]>>

    private var loadState: GemLoadState = .loading
    var sections: [ListSection<GemFiatTransactionRow>] {
        query.value
    }

    init(walletId: WalletId, service: any GemFiatQuoteServiceProtocol) {
        self.walletId = walletId
        self.service = service
        query = ObservableQuery(
            MappedQuery(FiatTransactionsQuery(walletId: walletId)) {
                DateSectionBuilder(items: fiatTransactionRows(data: $0.map { $0.toGem() }), dateKeyPath: \.createdAt).build()
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

    var emptyContentModel: EmptyStateViewModel {
        EmptyStateViewModel(kind: .activity)
    }

    func load() async {
        loadState = await service.refreshTransactions(hasTransactions: !sections.isEmpty)
    }
}
