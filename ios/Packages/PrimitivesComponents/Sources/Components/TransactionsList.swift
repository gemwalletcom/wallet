// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import SwiftUI

public struct TransactionsList: View {
    let transactions: [Primitives.TransactionExtended]
    let showSections: Bool
    let currency: String

    private var sections: [ListSection<Primitives.TransactionExtended>] {
        DateSectionBuilder(items: transactions, dateKeyPath: \.transaction.createdAt).build()
    }

    public init(
        _ transactions: [Primitives.TransactionExtended],
        currency: String,
        showSections: Bool = true,
    ) {
        self.transactions = transactions
        self.currency = currency
        self.showSections = showSections
    }

    public var body: some View {
        if showSections {
            ForEach(sections) { section in
                Section {
                    TransactionsListView(transactions: section.values, currency: currency)
                } header: {
                    section.title.map { Text($0) }
                }
            }
        } else {
            TransactionsListView(transactions: transactions, currency: currency)
        }
    }
}

private struct TransactionsListView: View {
    let transactions: [Primitives.TransactionExtended]
    let currency: String

    var body: some View {
        ForEach(transactions) { transaction in
            NavigationLink(value: Scenes.Transaction(transaction: transaction)) {
                TransactionView(model: .init(transaction: transaction, currency: currency))
            }
        }
    }
}
