// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.fiatTransactionListPhase
import func Gemstone.fiatTransactionRows
import protocol Gemstone.GemFiatQuoteServiceProtocol
import struct Gemstone.GemFiatTransactionRow
import enum Gemstone.GemListPhase
import enum Gemstone.GemLoadState
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

    public let query: ObservableQuery<MappedQuery<FiatTransactionsQuery, [ListSection<GemFiatTransactionRow>]>>

    private var loadState: GemLoadState = .loading
    var sections: [ListSection<GemFiatTransactionRow>] {
        query.value
    }

    init(walletId: WalletId, service: any GemFiatQuoteServiceProtocol) {
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

    var phase: GemListPhase {
        fiatTransactionListPhase(rows: sections.flatMap(\.values), state: loadState)
    }

    func load() async {
        loadState = await service.refreshTransactions()
    }
}
