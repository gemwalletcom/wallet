// Copyright (c). Gem Wallet. All rights reserved.

import func Gemstone.transactionFilters
import Primitives
import Testing
import Transactions

struct TransactionTypesFilterViewModelTests {
    @Test
    func listsCoreFiltersAndSelectsTheirTypes() {
        var model = TransactionTypesFilterViewModel()

        #expect(model.allTransactionsTypes == transactionFilters())
        #expect(model.requestFilters.isEmpty)

        model.selectedTypes = model.allTransactionsTypes
        #expect(Set(model.requestFilters) == Set(TransactionType.allCases))
        #expect(model.requestFilters.count == TransactionType.allCases.count)
    }
}
