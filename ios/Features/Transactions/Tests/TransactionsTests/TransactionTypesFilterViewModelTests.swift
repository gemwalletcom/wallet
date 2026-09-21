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

        model.selectedTypes = model.allTransactionsTypes
        #expect(model.isAnySelected)
    }
}
