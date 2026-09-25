// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import Primitives
import Testing
import Transactions

struct TransactionTypesFilterViewModelTests {
    @Test
    func listsCoreFiltersAndSelectsTheirTypes() {
        var model = TransactionTypesFilterViewModel()

        #expect(model.allTransactionsTypes == GemConstants.transactionFilters)

        model.selectedTypes = model.allTransactionsTypes
        #expect(model.isAnySelected)
    }
}
