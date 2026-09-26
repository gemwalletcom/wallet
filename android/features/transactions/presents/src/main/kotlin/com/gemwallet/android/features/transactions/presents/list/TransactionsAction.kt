package com.gemwallet.android.features.transactions.presents.list

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionId
import uniffi.gemstone.GemTransactionFilter

internal sealed interface TransactionsAction {
    data object Refresh : TransactionsAction
    data object ClearFilters : TransactionsAction
    data object Buy : TransactionsAction
    data object Receive : TransactionsAction
    data class OpenTransaction(val transactionId: TransactionId) : TransactionsAction
    data class SelectChainsFilter(val chains: List<Chain>) : TransactionsAction
    data class SelectTypesFilter(val types: List<GemTransactionFilter>) : TransactionsAction
}
