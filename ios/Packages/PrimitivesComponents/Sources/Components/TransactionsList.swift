// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemExplorerServiceProtocol
import class Gemstone.GemTransactionFormatter
import Components
import Primitives
import SwiftUI

public struct TransactionsList: View {
    let transactions: [Primitives.TransactionExtended]
    let showSections: Bool
    let explorerService: any GemExplorerServiceProtocol
    let transactionFormatter: GemTransactionFormatter
    let currency: String

    private var sections: [ListSection<Primitives.TransactionExtended>] {
        DateSectionBuilder(items: transactions, dateKeyPath: \.transaction.createdAt).build()
    }

    public init(
        explorerService: any GemExplorerServiceProtocol,
        transactionFormatter: GemTransactionFormatter,
        _ transactions: [Primitives.TransactionExtended],
        currency: String,
        showSections: Bool = true,
    ) {
        self.explorerService = explorerService
        self.transactionFormatter = transactionFormatter
        self.transactions = transactions
        self.currency = currency
        self.showSections = showSections
    }

    public var body: some View {
        if showSections {
            ForEach(sections) { section in
                Section {
                    TransactionsListView(
                        explorerService: explorerService,
                        transactionFormatter: transactionFormatter,
                        transactions: section.values,
                        currency: currency,
                    )
                } header: {
                    section.title.map { Text($0) }
                }
            }
        } else {
            TransactionsListView(
                explorerService: explorerService,
                transactionFormatter: transactionFormatter,
                transactions: transactions,
                currency: currency,
            )
        }
    }
}

private struct TransactionsListView: View {
    let transactions: [Primitives.TransactionExtended]
    let explorerService: any GemExplorerServiceProtocol
    let transactionFormatter: GemTransactionFormatter
    let currency: String

    init(explorerService: any GemExplorerServiceProtocol,
         transactionFormatter: GemTransactionFormatter,
         transactions: [Primitives.TransactionExtended],
         currency: String)
    {
        self.explorerService = explorerService
        self.transactionFormatter = transactionFormatter
        self.transactions = transactions
        self.currency = currency
    }

    var body: some View {
        ForEach(transactions) { transaction in
            NavigationLink(value: Scenes.Transaction(transaction: transaction)) {
                TransactionView(
                    model: .init(
                        explorerService: explorerService,
                        transactionFormatter: transactionFormatter,
                        transaction: transaction,
                        currency: currency,
                    ),
                )
            }
        }
    }
}
