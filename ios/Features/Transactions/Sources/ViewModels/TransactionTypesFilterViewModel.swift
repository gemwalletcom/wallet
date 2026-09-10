// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemTransactionFilter
import func Gemstone.transactionFilters
import Primitives

public struct TransactionTypesFilterViewModel: Equatable {
    public let allTransactionsTypes: [GemTransactionFilter]
    public var selectedTypes: [GemTransactionFilter]

    public init() {
        allTransactionsTypes = transactionFilters()
        selectedTypes = []
    }

    public var requestFilters: [TransactionType] {
        selectedTypes.flatMap { $0.transactionTypes().map { $0.map() } }
    }

    public var typeModel: TransactionsFilterTypeViewModel {
        TransactionsFilterTypeViewModel(
            type: TransactionsFilterType(selectedTypes: selectedTypes),
        )
    }

    public var isAnySelected: Bool {
        !selectedTypes.isEmpty
    }

    public var isEmpty: Bool {
        allTransactionsTypes.isEmpty
    }
}
