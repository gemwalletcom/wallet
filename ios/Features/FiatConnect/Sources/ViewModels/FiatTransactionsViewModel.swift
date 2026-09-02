// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemFiatQuoteServiceProtocol
import Components
import GemstoneServices
import Foundation
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
    var transactions: [FiatTransactionAssetData] {
        query.value
    }

    var sections: [ListSection<FiatTransactionAssetData>] {
        DateSectionBuilder(items: transactions, dateKeyPath: \.transaction.createdAt).build()
    }

    init(walletId: WalletId, service: any GemFiatQuoteServiceProtocol) {
        self.walletId = walletId
        self.service = service
        query = ObservableQuery(FiatTransactionsRequest(walletId: walletId), initialValue: [])
    }

    var title: String {
        Localized.Activity.title
    }

    var emptyContentModel: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: .activity(isViewOnly: false))
    }

    func load() async {
        do {
            try await service.syncTransactions()
        } catch {
            debugLog("FiatTransactionsViewModel load error: \(error)")
        }
    }
}
