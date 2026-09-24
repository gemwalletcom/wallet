// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemTransactionFilter
import func Gemstone.transactionsFilterSummary
import GemstonePrimitives
import Primitives

public struct TransactionTypesFilterViewModel: Equatable {
    public let allTransactionsTypes: [GemTransactionFilter]
    public var selectedTypes: [GemTransactionFilter]

    public init() {
        allTransactionsTypes = GemConstants.transactionFilters
        selectedTypes = []
    }

    public var typeModel: TransactionsFilterTypeViewModel {
        TransactionsFilterTypeViewModel(summary: transactionsFilterSummary(filters: selectedTypes))
    }

    public var isAnySelected: Bool {
        !selectedTypes.isEmpty
    }
}
