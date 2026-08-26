// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemFiatServiceProtocol
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
    private let service: any GemFiatServiceProtocol
    let walletId: WalletId

    public let query: ObservableQuery<FiatTransactionsRequest>
    var transactions: [FiatTransactionAssetData] {
        query.value
    }

    var sections: [ListSection<FiatTransactionAssetData>] {
        DateSectionBuilder(items: transactions, dateKeyPath: \.transaction.createdAt).build()
    }

    public init(walletId: WalletId, service: any GemFiatServiceProtocol) {
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

    func fetch() async {
        do {
            try await service.syncTransactions(walletId: walletId.id)
        } catch {
            debugLog("FiatTransactionsViewModel fetch error: \(error)")
        }
    }
}
