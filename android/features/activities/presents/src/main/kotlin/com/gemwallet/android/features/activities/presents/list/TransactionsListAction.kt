package com.gemwallet.android.features.activities.presents.list

import uniffi.gemstone.GemTransactionFilter
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionId

internal sealed interface TransactionsListAction {
    data object Refresh : TransactionsListAction
    data object ClearChainsFilter : TransactionsListAction
    data object ClearTypesFilter : TransactionsListAction
    data object Buy : TransactionsListAction
    data object Receive : TransactionsListAction
    data class OpenTransaction(val transactionId: TransactionId) : TransactionsListAction
    data class ApplyChainsFilter(val chains: List<Chain>) : TransactionsListAction
    data class ApplyTypesFilter(val types: List<GemTransactionFilter>) : TransactionsListAction
}
